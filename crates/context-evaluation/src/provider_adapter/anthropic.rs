#![forbid(unsafe_code)]

use super::{
    ProviderOutcome, add_usage, decode_response, dispatch_tool, parse_answer, system_instructions,
    tool_definitions, user_content,
};
use crate::agent_eval::{AdapterRequest, Usage};
use crate::production_adapter::RepositoryToolBoundary;
use reqwest::blocking::Client;
use serde_json::{Map, Value, json};

const ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const MAX_OUTPUT_TOKENS: u64 = 16_384;

pub(super) fn execute(
    client: &Client,
    key: &str,
    request: &AdapterRequest,
    tools: &mut RepositoryToolBoundary,
) -> Result<ProviderOutcome, String> {
    let mut messages = vec![json!({
        "role": "user",
        "content": user_content(request),
    })];
    let mut usage = Usage::default();
    for turn in 0..request.turn_limit {
        let allow_tools = turn + 1 < request.turn_limit;
        let body = request_body(request, &messages, allow_tools);
        let response = client
            .post(ENDPOINT)
            .header("x-api-key", key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|_| "send Anthropic Messages API request".to_owned())?;
        let value = decode_response(response)?;
        validate_response_model(&value, &request.model_identifier)?;
        add_usage(&mut usage, &parse_usage(&value)?);
        let content = value
            .get("content")
            .and_then(Value::as_array)
            .ok_or_else(|| "Anthropic response omitted content".to_owned())?;
        let calls = tool_calls(content)?;
        validate_stop_reason(&value, !calls.is_empty())?;
        if calls.is_empty() {
            let text = output_text(content)?;
            return Ok(ProviderOutcome {
                answer: parse_answer(&text)?,
                usage,
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
    }
    Err("Anthropic adapter exhausted the fixed turn limit".to_owned())
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
        "max_tokens": MAX_OUTPUT_TOKENS,
        "output_config": {
            "effort": "high",
            "format": evaluation_output_format(),
        },
        "tools": anthropic_tools(),
        "tool_choice": tool_choice,
    })
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
    use crate::agent_eval::{Arm, PricingSchedule};

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
            pricing_schedule: PricingSchedule::default(),
            container_image: "test".into(),
            operation_timestamp: "1970-01-01T00:00:00Z".into(),
            turn_limit: 3,
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
}
