#![forbid(unsafe_code)]

use super::{
    Provider, ProviderOutcome, add_usage, decode_response, dispatch_tool, emit_progress,
    parse_answer, request_timeout, system_instructions, tool_definitions,
};
use crate::agent_eval::{AdapterRequest, ProgressStage, Usage};
use crate::production_adapter::RepositoryToolBoundary;
use reqwest::blocking::{Client, Response};
use serde_json::{Map, Value, json};
use std::io::{BufRead, BufReader, Read};
use std::time::Instant;

const ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const TOKEN_COUNT_ENDPOINT: &str = "https://api.anthropic.com/v1/messages/count_tokens";
const API_VERSION: &str = "2023-06-01";

pub(super) fn execute(
    client: &Client,
    key: &str,
    request: &AdapterRequest,
    prepared_user_content: &str,
    tools: &mut RepositoryToolBoundary,
    started: Instant,
) -> Result<ProviderOutcome, String> {
    let mut messages = initial_messages(prepared_user_content);
    let mut usage = Usage::default();
    for turn in 0..request.turn_limit {
        let allow_tools = turn + 1 < request.turn_limit;
        let body = request_body(request, &messages, allow_tools);
        emit_progress(
            Provider::Anthropic,
            request,
            started,
            ProgressStage::ProviderRequestStarted,
            turn + 1,
            None,
            None,
            None,
            None,
            None,
        )?;
        let response = client
            .post(ENDPOINT)
            .header("x-api-key", key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .timeout(request_timeout(request, started)?)
            .send()
            .map_err(|_| "send Anthropic Messages API request".to_owned())?;
        let value = decode_stream_response(response)?;
        validate_response_model(&value, &request.model_identifier)?;
        let request_usage = parse_usage(&value)?;
        add_usage(&mut usage, &request_usage);
        let content = value
            .get("content")
            .and_then(Value::as_array)
            .ok_or_else(|| "Anthropic response omitted content".to_owned())?;
        let calls = tool_calls(content)?;
        validate_stop_reason(&value, !calls.is_empty())?;
        let stop_reason = value
            .get("stop_reason")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        emit_progress(
            Provider::Anthropic,
            request,
            started,
            ProgressStage::ProviderResponseCompleted,
            turn + 1,
            None,
            None,
            Some(request_usage),
            Some(stop_reason),
            Some(u64::try_from(calls.len()).unwrap_or(u64::MAX)),
        )?;
        if calls.is_empty() {
            let text = output_text(content)?;
            return Ok(ProviderOutcome {
                answer: parse_answer(&text)?,
                usage,
                completed_turns: turn + 1,
            });
        }
        messages.push(json!({"role": "assistant", "content": content}));
        let results = calls
            .into_iter()
            .map(|call| {
                let result = dispatch_tool(&call.name, &call.arguments, tools);
                let is_error = serde_json::from_str::<Value>(&result)
                    .ok()
                    .and_then(|value| value.get("error").cloned())
                    .is_some();
                json!({
                    "type": "tool_result",
                    "tool_use_id": call.id,
                    "content": result,
                    "is_error": is_error,
                })
            })
            .collect::<Vec<_>>();
        messages.push(json!({"role": "user", "content": results}));
        emit_progress(
            Provider::Anthropic,
            request,
            started,
            ProgressStage::ToolsDispatched,
            turn + 1,
            None,
            None,
            None,
            None,
            None,
        )?;
    }
    Err("Anthropic adapter exhausted the fixed turn limit".to_owned())
}

pub(super) fn count_tokens(
    client: &Client,
    key: &str,
    request: &AdapterRequest,
    prepared_user_content: &str,
    started: Instant,
) -> Result<u64, String> {
    let mut body = request_body(request, &initial_messages(prepared_user_content), true);
    let body = body
        .as_object_mut()
        .ok_or_else(|| "Anthropic token-count request was invalid".to_owned())?;
    body.remove("stream");
    body.remove("max_tokens");
    let response = client
        .post(TOKEN_COUNT_ENDPOINT)
        .header("x-api-key", key)
        .header("anthropic-version", API_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .timeout(request_timeout(request, started)?)
        .send()
        .map_err(|_| "send Anthropic Token Counting API request".to_owned())?;
    decode_response(response)?
        .get("input_tokens")
        .and_then(Value::as_u64)
        .filter(|value| *value > 0)
        .ok_or_else(|| "Anthropic token count omitted input_tokens".to_owned())
}

pub(super) fn initial_messages(prepared_user_content: &str) -> Vec<Value> {
    vec![json!({
        "role": "user",
        "content": prepared_user_content,
    })]
}

fn request_body(request: &AdapterRequest, messages: &[Value], allow_tools: bool) -> Value {
    let tool_choice = if allow_tools {
        json!({"type": "auto", "disable_parallel_tool_use": true})
    } else {
        json!({"type": "none"})
    };
    json!({
        "model": request.model_identifier,
        "system": system_instructions(),
        "messages": messages,
        "max_tokens": request.provider_max_output_tokens,
        "output_config": {
            "effort": request.provider_effort,
            "format": evaluation_output_format(),
        },
        "tools": anthropic_tools(),
        "tool_choice": tool_choice,
        "stream": true,
    })
}

const MAX_SSE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_SSE_EVENT_BYTES: usize = 256 * 1024;

fn decode_stream_response(response: Response) -> Result<Value, String> {
    let status = response.status();
    if !status.is_success() {
        return Err(format!("provider request failed with HTTP {status}"));
    }
    parse_sse(BufReader::new(response.take(MAX_SSE_BYTES + 1)))
}

fn parse_sse<R: BufRead>(mut reader: R) -> Result<Value, String> {
    let mut line = String::new();
    let mut event_name = String::new();
    let mut data = String::new();
    let mut consumed = 0_u64;
    let mut message: Option<Value> = None;
    let mut partial_inputs = std::collections::BTreeMap::<usize, String>::new();
    let mut stopped = false;
    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|_| "read Anthropic SSE stream".to_owned())?;
        if read == 0 {
            break;
        }
        consumed = consumed.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
        if consumed > MAX_SSE_BYTES {
            return Err("Anthropic SSE stream exceeded byte limit".to_owned());
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            if !data.is_empty() {
                apply_sse_event(
                    &event_name,
                    &data,
                    &mut message,
                    &mut partial_inputs,
                    &mut stopped,
                )?;
            }
            event_name.clear();
            data.clear();
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("event:") {
            value.trim().clone_into(&mut event_name);
        } else if let Some(value) = trimmed.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value.trim_start());
            if data.len() > MAX_SSE_EVENT_BYTES {
                return Err("Anthropic SSE event exceeded byte limit".to_owned());
            }
        } else if !trimmed.starts_with(':') {
            return Err("Anthropic SSE stream contained an invalid field".to_owned());
        }
    }
    if !data.is_empty() {
        apply_sse_event(
            &event_name,
            &data,
            &mut message,
            &mut partial_inputs,
            &mut stopped,
        )?;
    }
    if !stopped || !partial_inputs.is_empty() {
        return Err("Anthropic SSE stream ended before message completion".to_owned());
    }
    message.ok_or_else(|| "Anthropic SSE stream omitted message_start".to_owned())
}

fn apply_sse_event(
    event_name: &str,
    data: &str,
    message: &mut Option<Value>,
    partial_inputs: &mut std::collections::BTreeMap<usize, String>,
    stopped: &mut bool,
) -> Result<(), String> {
    let event: Value =
        serde_json::from_str(data).map_err(|_| "parse Anthropic SSE event".to_owned())?;
    let event_type = event
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| "Anthropic SSE event omitted type".to_owned())?;
    if !event_name.is_empty() && event_name != event_type {
        return Err("Anthropic SSE event name disagreed with data".to_owned());
    }
    match event_type {
        "ping" => Ok(()),
        "error" => Err("Anthropic SSE stream reported an error".to_owned()),
        "message_start" => {
            if message.is_some() || *stopped {
                return Err("duplicate Anthropic message_start".to_owned());
            }
            *message = Some(
                event
                    .get("message")
                    .cloned()
                    .ok_or_else(|| "Anthropic message_start omitted message".to_owned())?,
            );
            Ok(())
        }
        "content_block_start" => {
            let target = message
                .as_mut()
                .ok_or_else(|| "Anthropic content preceded message_start".to_owned())?;
            let index = event_index(&event)?;
            let content = target
                .get_mut("content")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| "Anthropic streaming message omitted content".to_owned())?;
            if index != content.len() {
                return Err("Anthropic content block index was out of order".to_owned());
            }
            let block = event
                .get("content_block")
                .cloned()
                .ok_or_else(|| "Anthropic content block start omitted block".to_owned())?;
            if block.get("type").and_then(Value::as_str) == Some("tool_use") {
                partial_inputs.insert(index, String::new());
            }
            content.push(block);
            Ok(())
        }
        "content_block_delta" => apply_content_delta(message, partial_inputs, &event),
        "content_block_stop" => finish_content_block(message, partial_inputs, &event),
        "message_delta" => apply_message_delta(message, &event),
        "message_stop" => {
            if message.is_none() || *stopped {
                return Err("unexpected Anthropic message_stop".to_owned());
            }
            *stopped = true;
            Ok(())
        }
        _ => Err("Anthropic SSE stream contained an unexpected event".to_owned()),
    }
}

fn event_index(event: &Value) -> Result<usize, String> {
    event
        .get("index")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| "Anthropic SSE event had an invalid block index".to_owned())
}

fn apply_content_delta(
    message: &mut Option<Value>,
    partial_inputs: &mut std::collections::BTreeMap<usize, String>,
    event: &Value,
) -> Result<(), String> {
    let index = event_index(event)?;
    let delta = event
        .get("delta")
        .ok_or_else(|| "Anthropic content delta omitted delta".to_owned())?;
    let kind = delta.get("type").and_then(Value::as_str).unwrap_or("");
    if kind == "input_json_delta" {
        let partial = delta
            .get("partial_json")
            .and_then(Value::as_str)
            .ok_or_else(|| "Anthropic tool delta omitted partial JSON".to_owned())?;
        let target = partial_inputs
            .get_mut(&index)
            .ok_or_else(|| "Anthropic tool delta targeted a non-tool block".to_owned())?;
        target.push_str(partial);
        if target.len() > MAX_SSE_EVENT_BYTES {
            return Err("Anthropic tool input exceeded byte limit".to_owned());
        }
        return Ok(());
    }
    let (field, addition) = match kind {
        "text_delta" => ("text", "text"),
        "thinking_delta" => ("thinking", "thinking"),
        "signature_delta" => ("signature", "signature"),
        _ => return Err("Anthropic SSE stream contained an unexpected delta".to_owned()),
    };
    let addition = delta
        .get(addition)
        .and_then(Value::as_str)
        .ok_or_else(|| "Anthropic content delta omitted text".to_owned())?;
    let block = message
        .as_mut()
        .and_then(|value| value.get_mut("content"))
        .and_then(Value::as_array_mut)
        .and_then(|content| content.get_mut(index))
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "Anthropic content delta targeted a missing block".to_owned())?;
    let current = block
        .entry(field)
        .or_insert_with(|| Value::String(String::new()));
    let current = current
        .as_str()
        .ok_or_else(|| "Anthropic content delta targeted a non-string field".to_owned())?;
    let mut combined = current.to_owned();
    combined.push_str(addition);
    if combined.len() > MAX_SSE_EVENT_BYTES {
        return Err("Anthropic content block exceeded byte limit".to_owned());
    }
    block.insert(field.into(), Value::String(combined));
    Ok(())
}

fn finish_content_block(
    message: &mut Option<Value>,
    partial_inputs: &mut std::collections::BTreeMap<usize, String>,
    event: &Value,
) -> Result<(), String> {
    let index = event_index(event)?;
    if let Some(partial) = partial_inputs.remove(&index)
        && !partial.is_empty()
    {
        let input: Value = serde_json::from_str(&partial)
            .map_err(|_| "Anthropic tool input was invalid JSON".to_owned())?;
        let block = message
            .as_mut()
            .and_then(|value| value.get_mut("content"))
            .and_then(Value::as_array_mut)
            .and_then(|content| content.get_mut(index))
            .and_then(Value::as_object_mut)
            .ok_or_else(|| "Anthropic content stop targeted a missing block".to_owned())?;
        block.insert("input".into(), input);
    }
    Ok(())
}

fn apply_message_delta(message: &mut Option<Value>, event: &Value) -> Result<(), String> {
    let target = message
        .as_mut()
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "Anthropic message delta preceded message_start".to_owned())?;
    let delta = event
        .get("delta")
        .and_then(Value::as_object)
        .ok_or_else(|| "Anthropic message delta omitted delta".to_owned())?;
    for (key, value) in delta {
        target.insert(key.clone(), value.clone());
    }
    if let Some(addition) = event.get("usage").and_then(Value::as_object) {
        let usage = target
            .entry("usage")
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .ok_or_else(|| "Anthropic streaming usage was invalid".to_owned())?;
        for (key, value) in addition {
            usage.insert(key.clone(), value.clone());
        }
    }
    Ok(())
}

fn anthropic_tools() -> Value {
    let mut tools = tool_definitions();
    for tool in tools
        .as_array_mut()
        .expect("shared tool definitions are an array")
    {
        tool.as_object_mut()
            .expect("shared tool definitions contain objects")
            .insert("strict".to_owned(), Value::Bool(true));
    }
    let properties = tools[1]["input_schema"]["properties"]
        .as_object_mut()
        .expect("read tool properties are an object");
    for field in ["line_start", "line_end"] {
        let schema = properties[field]
            .as_object_mut()
            .expect("line range schemas are objects");
        schema.remove("minimum");
        schema.insert(
            "description".to_owned(),
            Value::String("An integer greater than or equal to 1.".to_owned()),
        );
    }
    tools
}

fn evaluation_output_format() -> Value {
    json!({
        "type": "json_schema",
        "schema": {
            "type": "object",
            "properties": {
                "answer": {"type": "string"},
                "evidence": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"},
                            "line_start": {"type": "integer"},
                            "line_end": {"type": "integer"},
                        },
                        "required": ["path", "line_start", "line_end"],
                        "additionalProperties": false,
                    },
                },
            },
            "required": ["answer", "evidence"],
            "additionalProperties": false,
        },
    })
}

struct ToolCall {
    id: String,
    name: String,
    arguments: String,
}

fn tool_calls(content: &[Value]) -> Result<Vec<ToolCall>, String> {
    content
        .iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("tool_use"))
        .map(|block| {
            let input = block
                .get("input")
                .ok_or_else(|| "Anthropic tool call omitted input".to_owned())?;
            Ok(ToolCall {
                id: required_string(block, "id")?.to_owned(),
                name: required_string(block, "name")?.to_owned(),
                arguments: input.to_string(),
            })
        })
        .collect()
}

fn output_text(content: &[Value]) -> Result<String, String> {
    let parts = content
        .iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|block| block.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>();
    if parts.is_empty() {
        Err("Anthropic response omitted final output text".to_owned())
    } else {
        Ok(parts.join(""))
    }
}

fn parse_usage(response: &Value) -> Result<Usage, String> {
    let usage = response
        .get("usage")
        .and_then(Value::as_object)
        .ok_or_else(|| "Anthropic response omitted usage".to_owned())?;
    let uncached = required_u64(usage, "input_tokens")?;
    let cached = optional_u64(usage, "cache_read_input_tokens");
    let cache_write = optional_u64(usage, "cache_creation_input_tokens");
    let input_tokens = uncached.saturating_add(cached).saturating_add(cache_write);
    let output_tokens = required_u64(usage, "output_tokens")?;
    Ok(Usage {
        input_tokens,
        output_tokens,
        total_tokens: input_tokens.saturating_add(output_tokens),
        estimated_cost_usd: 0.0,
        cached_input_tokens: cached,
        cache_write_input_tokens: cache_write,
        reasoning_tokens: 0,
        provider_requests: 1,
        tool_calls: 0,
        repository_file_reads: 0,
        repeated_repository_file_reads: 0,
    })
}

fn validate_response_model(response: &Value, expected: &str) -> Result<(), String> {
    if response.get("model").and_then(Value::as_str) == Some(expected) {
        Ok(())
    } else {
        Err("Anthropic returned a different model identifier".to_owned())
    }
}

fn validate_stop_reason(response: &Value, has_tool_calls: bool) -> Result<(), String> {
    let expected = if has_tool_calls {
        "tool_use"
    } else {
        "end_turn"
    };
    if response.get("stop_reason").and_then(Value::as_str) == Some(expected) {
        Ok(())
    } else {
        Err("Anthropic response ended with an unexpected stop reason".to_owned())
    }
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("Anthropic content field {key:?} is invalid"))
}

fn required_u64(value: &Map<String, Value>, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Anthropic usage field {key:?} is invalid"))
}

fn optional_u64(value: &Map<String, Value>, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_eval::{Arm, PacketResourceSpec, PricingSchedule};

    fn request() -> AdapterRequest {
        AdapterRequest {
            task_id: "task".into(),
            prompt: "question".into(),
            arm: Arm::BaselineA,
            workspace_root: "/tmp/source".into(),
            source_files: vec!["one.rs".into()],
            context_plan: Vec::new(),
            source_fingerprint_sha256: "sha256:test".into(),
            model_identifier: "claude-opus-5".into(),
            model_context_renderer_identifier: "impresari-evaluation-model-context".into(),
            model_context_renderer_version: "1.0.0".into(),
            max_rendered_context_bytes: 131_072,
            pricing_schedule: PricingSchedule::default(),
            container_image: "test".into(),
            operation_timestamp: "1970-01-01T00:00:00Z".into(),
            turn_limit: 3,
            provider_effort: "high".into(),
            provider_max_output_tokens: 16_384,
            provider_request_timeout_seconds: 120,
            command_timeout_seconds: 600,
            packet_resource_policy: PacketResourceSpec {
                requested_bytes: 65_536,
                max_evidence_items: 100,
                max_files: 10_000,
                max_excerpt_bytes_per_item: 4096,
                max_matches: 1000,
                max_traversal_depth: 32,
                max_elapsed_ms: 30_000,
                max_memory_bytes: 536_870_912,
            },
            packet: None,
        }
    }

    #[test]
    fn request_is_high_effort_without_cache_or_conversation_state() {
        let body = request_body(
            &request(),
            &[json!({"role":"user","content":"question"})],
            true,
        );
        assert_eq!(body["output_config"]["effort"], "high");
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert_eq!(
            body["output_config"]["format"]["schema"]["required"],
            json!(["answer", "evidence"])
        );
        assert_eq!(
            body["output_config"]["format"]["schema"]["additionalProperties"],
            false
        );
        assert_eq!(body["tool_choice"]["disable_parallel_tool_use"], true);
        assert_eq!(body["tools"][0]["strict"], true);
        assert_eq!(body["stream"], true);
        assert!(
            body["tools"][1]["input_schema"]["properties"]["line_start"]
                .get("minimum")
                .is_none()
        );
        assert!(body.get("cache_control").is_none());
        assert!(body.get("conversation").is_none());
    }

    #[test]
    fn final_bounded_turn_disables_tools() {
        let body = request_body(
            &request(),
            &[json!({"role":"user","content":"question"})],
            false,
        );
        assert_eq!(body["tool_choice"], json!({"type": "none"}));
    }

    #[test]
    fn normalizes_provider_usage_and_parses_final_json() {
        let response = json!({
            "model": "claude-opus-5",
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 100,
                "cache_read_input_tokens": 0,
                "cache_creation_input_tokens": 0,
                "output_tokens": 20
            },
            "content": [{
                "type": "text",
                "text": "{\"answer\":\"ok\",\"evidence\":[{\"path\":\"one.rs\",\"line_start\":1,\"line_end\":1}]}"
            }]
        });
        validate_response_model(&response, "claude-opus-5").expect("model");
        validate_stop_reason(&response, false).expect("stop reason");
        let usage = parse_usage(&response).expect("usage");
        assert_eq!(usage.total_tokens, 120);
        let content = response["content"].as_array().expect("content");
        let answer = parse_answer(&output_text(content).expect("text")).expect("answer");
        assert_eq!(answer.answer, "ok");
    }

    #[test]
    fn reconstructs_streamed_text_usage_and_stop_reason() {
        let stream = concat!(
            "event: message_start\n",
            "data: {\"type\":\"message_start\",\"message\":{\"model\":\"claude-opus-5\",\"content\":[],\"stop_reason\":null,\"usage\":{\"input_tokens\":10}}}\n\n",
            "event: content_block_start\n",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"{\\\"answer\\\":\\\"ok\\\",\\\"evidence\\\":[]}\"}}\n\n",
            "event: content_block_stop\n",
            "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":5}}\n\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n\n"
        );
        let value = parse_sse(std::io::Cursor::new(stream)).expect("stream");
        validate_stop_reason(&value, false).expect("stop reason");
        assert_eq!(parse_usage(&value).expect("usage").total_tokens, 15);
        assert_eq!(
            output_text(value["content"].as_array().expect("content")).expect("text"),
            "{\"answer\":\"ok\",\"evidence\":[]}"
        );
    }

    #[test]
    fn reconstructs_partial_tool_json() {
        let stream = concat!(
            "event: message_start\n",
            "data: {\"type\":\"message_start\",\"message\":{\"model\":\"claude-opus-5\",\"content\":[],\"stop_reason\":null,\"usage\":{\"input_tokens\":10}}}\n\n",
            "event: content_block_start\n",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tool-1\",\"name\":\"read_repository_file\",\"input\":{}}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\":\\\"one.rs\\\",\"}}\n\n",
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"line_start\\\":1,\\\"line_end\\\":1}\"}}\n\n",
            "event: content_block_stop\n",
            "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":5}}\n\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n\n"
        );
        let value = parse_sse(std::io::Cursor::new(stream)).expect("stream");
        let calls = tool_calls(value["content"].as_array().expect("content")).expect("calls");
        assert_eq!(calls.len(), 1);
        assert!(calls[0].arguments.contains("one.rs"));
    }

    #[test]
    fn preserves_empty_tool_input_when_no_delta_is_emitted() {
        let mut message = Some(json!({
            "content": [{"type":"tool_use","id":"tool-1","name":"list_repository_files","input":{}}]
        }));
        let mut partial = std::collections::BTreeMap::from([(0_usize, String::new())]);
        finish_content_block(
            &mut message,
            &mut partial,
            &json!({"type":"content_block_stop","index":0}),
        )
        .expect("empty input");
        assert_eq!(message.expect("message")["content"][0]["input"], json!({}));
        assert!(partial.is_empty());
    }

    #[test]
    fn rejects_error_incomplete_and_oversized_sse() {
        let error = "event: error\ndata: {\"type\":\"error\",\"error\":{}}\n\n";
        assert!(parse_sse(std::io::Cursor::new(error)).is_err());
        let incomplete = "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"content\":[]}}\n\n";
        assert!(parse_sse(std::io::Cursor::new(incomplete)).is_err());
        let oversized = format!(
            "event: ping\ndata: {}\n\n",
            "x".repeat(MAX_SSE_EVENT_BYTES + 1)
        );
        assert!(parse_sse(std::io::Cursor::new(oversized)).is_err());
    }
}
