#![forbid(unsafe_code)]

use super::{
    ProviderOutcome, add_usage, decode_response, dispatch_tool, parse_answer, system_instructions,
    tool_definitions, user_content,
};
use crate::agent_eval::{AdapterRequest, Usage};
use crate::production_adapter::RepositoryToolBoundary;
use reqwest::blocking::Client;
use serde_json::{Map, Value, json};

const ENDPOINT: &str = "https://api.openai.com/v1/responses";
const MAX_OUTPUT_TOKENS: u64 = 16_384;
const STANDARD_RATE_MAX_INPUT_TOKENS: u64 = 272_000;

pub(super) fn execute(
    client: &Client,
    key: &str,
    request: &AdapterRequest,
    tools: &mut RepositoryToolBoundary,
) -> Result<ProviderOutcome, String> {
    let mut input = vec![
        json!({"role": "developer", "content": system_instructions()}),
        json!({"role": "user", "content": user_content(request)}),
    ];
    let mut usage = Usage::default();
    for _ in 0..request.turn_limit {
        let body = request_body(request, &input);
        let response = client
            .post(ENDPOINT)
            .bearer_auth(key)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|_| "send OpenAI Responses API request".to_owned())?;
        let value = decode_response(response)?;
        validate_response_model(&value, &request.model_identifier)?;
        validate_response_state(&value)?;
        let request_usage = parse_usage(&value)?;
        validate_standard_rate(&request_usage)?;
        add_usage(&mut usage, &request_usage);
        let output = value
            .get("output")
            .and_then(Value::as_array)
            .ok_or_else(|| "OpenAI response omitted output".to_owned())?;
        let calls = function_calls(output)?;
        if calls.is_empty() {
            let text = output_text(output)?;
            return Ok(ProviderOutcome {
                answer: parse_answer(&text)?,
                usage,
            });
        }
        input.extend(output.iter().cloned());
        for call in calls {
            let result = dispatch_tool(&call.name, &call.arguments, tools);
            input.push(json!({
                "type": "function_call_output",
                "call_id": call.call_id,
                "output": result,
            }));
        }
    }
    Err("OpenAI adapter exhausted the fixed turn limit".to_owned())
}

fn request_body(request: &AdapterRequest, input: &[Value]) -> Value {
    json!({
        "model": request.model_identifier,
        "store": false,
        "service_tier": "default",
        "include": ["reasoning.encrypted_content"],
        "reasoning": {"effort": "high", "context": "current_turn"},
        "prompt_cache_options": {"mode": "explicit"},
        "parallel_tool_calls": false,
        "max_output_tokens": MAX_OUTPUT_TOKENS,
        "tools": openai_tools(),
        "tool_choice": "auto",
        "input": input,
    })
}

fn openai_tools() -> Value {
    let tools = tool_definitions();
    Value::Array(
        tools
            .as_array()
            .expect("shared tool definitions are an array")
            .iter()
            .map(|tool| {
                json!({
                    "type": "function",
                    "name": tool["name"],
                    "description": tool["description"],
                    "parameters": tool["input_schema"],
                    "strict": true,
                })
            })
            .collect(),
    )
}

struct FunctionCall {
    call_id: String,
    name: String,
    arguments: String,
}

fn function_calls(output: &[Value]) -> Result<Vec<FunctionCall>, String> {
    output
        .iter()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("function_call"))
        .map(|item| {
            Ok(FunctionCall {
                call_id: required_string(item, "call_id")?.to_owned(),
                name: required_string(item, "name")?.to_owned(),
                arguments: required_string(item, "arguments")?.to_owned(),
            })
        })
        .collect()
}

fn output_text(output: &[Value]) -> Result<String, String> {
    let parts = output
        .iter()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("message"))
        .filter_map(|item| item.get("content").and_then(Value::as_array))
        .flatten()
        .filter(|part| part.get("type").and_then(Value::as_str) == Some("output_text"))
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect::<Vec<_>>();
    if parts.is_empty() {
        Err("OpenAI response omitted final output text".to_owned())
    } else {
        Ok(parts.join(""))
    }
}

fn parse_usage(response: &Value) -> Result<Usage, String> {
    let usage = response
        .get("usage")
        .and_then(Value::as_object)
        .ok_or_else(|| "OpenAI response omitted usage".to_owned())?;
    let input_tokens = required_u64(usage, "input_tokens")?;
    let output_tokens = required_u64(usage, "output_tokens")?;
    let input_details = usage.get("input_tokens_details").and_then(Value::as_object);
    let output_details = usage
        .get("output_tokens_details")
        .and_then(Value::as_object);
    Ok(Usage {
        input_tokens,
        output_tokens,
        total_tokens: input_tokens.saturating_add(output_tokens),
        estimated_cost_usd: 0.0,
        cached_input_tokens: optional_u64(input_details, "cached_tokens"),
        cache_write_input_tokens: optional_u64(input_details, "cache_write_tokens"),
        reasoning_tokens: optional_u64(output_details, "reasoning_tokens"),
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
        Err("OpenAI returned a different model identifier".to_owned())
    }
}

fn validate_response_state(response: &Value) -> Result<(), String> {
    if response.get("status").and_then(Value::as_str) != Some("completed") {
        return Err("OpenAI response did not complete".to_owned());
    }
    if response.get("service_tier").and_then(Value::as_str) != Some("default") {
        return Err("OpenAI returned a different service tier".to_owned());
    }
    Ok(())
}

fn validate_standard_rate(usage: &Usage) -> Result<(), String> {
    if usage.input_tokens > STANDARD_RATE_MAX_INPUT_TOKENS {
        Err("OpenAI request exceeded the frozen standard-rate token tier".to_owned())
    } else {
        Ok(())
    }
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("OpenAI output field {key:?} is invalid"))
}

fn required_u64(value: &Map<String, Value>, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("OpenAI usage field {key:?} is invalid"))
}

fn optional_u64(value: Option<&Map<String, Value>>, key: &str) -> u64 {
    value
        .and_then(|value| value.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0)
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
            model_identifier: "gpt-5.6-sol".into(),
            pricing_schedule: PricingSchedule::default(),
            container_image: "test".into(),
            operation_timestamp: "1970-01-01T00:00:00Z".into(),
            turn_limit: 3,
            packet: None,
        }
    }

    #[test]
    fn request_is_stateless_high_effort_and_cache_disabled() {
        let body = request_body(&request(), &[json!({"role":"user","content":"x"})]);
        assert_eq!(body["store"], false);
        assert_eq!(body["service_tier"], "default");
        assert_eq!(body["reasoning"]["effort"], "high");
        assert_eq!(body["reasoning"]["context"], "current_turn");
        assert_eq!(body["prompt_cache_options"]["mode"], "explicit");
        assert_eq!(body["parallel_tool_calls"], false);
        assert!(body.get("previous_response_id").is_none());
        assert!(body.get("conversation").is_none());
    }

    #[test]
    fn parses_provider_usage_and_final_json() {
        let response = json!({
            "model": "gpt-5.6-sol",
            "status": "completed",
            "service_tier": "default",
            "usage": {
                "input_tokens": 100,
                "output_tokens": 20,
                "input_tokens_details": {"cached_tokens": 0, "cache_write_tokens": 0},
                "output_tokens_details": {"reasoning_tokens": 5}
            },
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": "{\"answer\":\"ok\",\"evidence\":[{\"path\":\"one.rs\",\"line_start\":1,\"line_end\":1}]}"
                }]
            }]
        });
        validate_response_model(&response, "gpt-5.6-sol").expect("model");
        validate_response_state(&response).expect("response state");
        let usage = parse_usage(&response).expect("usage");
        validate_standard_rate(&usage).expect("standard rate");
        assert_eq!(usage.total_tokens, 120);
        assert_eq!(usage.reasoning_tokens, 5);
        let answer = parse_answer(
            &output_text(response["output"].as_array().expect("output")).expect("text"),
        )
        .expect("answer");
        assert_eq!(answer.answer, "ok");
        let mut long_context = usage;
        long_context.input_tokens = STANDARD_RATE_MAX_INPUT_TOKENS + 1;
        assert!(validate_standard_rate(&long_context).is_err());
    }
}
