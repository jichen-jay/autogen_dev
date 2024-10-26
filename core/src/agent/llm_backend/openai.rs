use async_openai::types::{
    ChatCompletionToolType, CreateChatCompletionResponse, FinishReason, Role,
};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    ClientBuilder,
};
use serde_json::Value;

use crate::agent::llm_backend::{LlmConfig, OPENAI_CONFIG};
use crate::msg_types::{llm_msg_types::LlmMessage, new_agent_id, RequestUsage};
use crate::tool_types::FunctionCallInput;

pub async fn chat_wrapper_openai(
    llm_config: &LlmConfig,
    system_prompt: &str,
    input: &str,
    max_token: u16,
) -> anyhow::Result<(LlmMessage, RequestUsage)> {
    let mut headers = HeaderMap::new();
    let api_key = std::env::var(llm_config.api_key_str)?;
    let bearer_token = format!("Bearer {}", api_key);

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("MyClient/1.0.0"));
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token)?);

    let messages = serde_json::json!([
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": input}
    ]);

    let uri = llm_config.base_url;

    let body_json = serde_json::json!({
        "model": llm_config.model, // Ensure this is a model that supports function calling
        "messages": messages,
        "max_tokens": max_token,
        "temperature": 0.3
    });

    let body = serde_json::to_vec(&body_json)?;

    let client = ClientBuilder::new().default_headers(headers).build()?;

    match client.post(uri).body(body.clone()).send().await {
        Ok(chat) => {
            let response_body = chat.text().await?;
            println!("coding response_body: {:?}", response_body.clone());

            let raw_output: CreateChatCompletionResponse =
                serde_json::from_str::<CreateChatCompletionResponse>(&response_body)?;

            if let Some(choice) = raw_output.choices.first() {
                let message = &choice.message;
                let role = &message.role;
                let finish_reason = choice.finish_reason.unwrap();

                let usage = raw_output
                    .usage
                    .map(|u| RequestUsage {
                        prompt_tokens: u.prompt_tokens as i32,
                        completion_tokens: u.completion_tokens as i32,
                    })
                    .unwrap_or(RequestUsage {
                        prompt_tokens: 0,
                        completion_tokens: 0,
                    });

                let llm_message =
                    LlmMessage::assistant_text(message.clone().content.unwrap(), new_agent_id());
                Ok((llm_message, usage))
            } else {
                Err(anyhow::anyhow!("Could not convert to LlmMessage"))
            }
        }
        Err(_e) => Err(anyhow::anyhow!("Failed to get reply from OpenAI: {:?}", _e)),
    }
}
pub async fn chat_inner_async_wrapper(
    llm_config: &LlmConfig,
    functions: &serde_json::Value,
    system_prompt: &str,
    input: &str,
    max_token: u16,
) -> anyhow::Result<(LlmMessage, RequestUsage)> {
    let mut headers = HeaderMap::new();
    let api_key = std::env::var(llm_config.api_key_str)?;
    let bearer_token = format!("Bearer {}", api_key);

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("MyClient/1.0.0"));
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token)?);

    let messages = serde_json::json!([
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": input}
    ]);

    let uri = llm_config.base_url;

    let body_json = serde_json::json!({
        "model": llm_config.model, // Ensure this is a model that supports function calling
        "messages": messages,
        "functions": functions,
        "function_call": "auto", // or specify the function name
        "max_tokens": max_token,
        "temperature": 0.3
    });

    let body = serde_json::to_vec(&body_json)?;

    let client = ClientBuilder::new().default_headers(headers).build()?;

    match client.post(uri).body(body.clone()).send().await {
        Ok(chat) => {
            let response_body = chat.text().await?;
            println!("coding response_body: {:?}", response_body.clone());

            let raw_output: CreateChatCompletionResponse =
                serde_json::from_str::<CreateChatCompletionResponse>(&response_body)?;

            if let Some(choice) = raw_output.choices.first() {
                let message = &choice.message;
                let role = &message.role;
                let finish_reason = choice.finish_reason.unwrap();
                let llm_message = match role {
                    Role::Assistant | Role::Function => {
                        if finish_reason == FinishReason::ToolCalls
                            || finish_reason == FinishReason::Stop
                        {
                            if let Some(tool_calls) = &message.tool_calls {
                                let function_calls: Vec<FunctionCallInput> = tool_calls
                                    .iter()
                                    .filter(|tool_call| {
                                        tool_call.r#type == ChatCompletionToolType::Function
                                    })
                                    .filter_map(|tool_call| {
                                        serde_json::from_str::<Value>(&tool_call.function.arguments)
                                            .ok()
                                            .map(|arguments_obj| FunctionCallInput {
                                                function_name: tool_call.function.name.clone(),
                                                arguments_obj,
                                                return_type: "".to_string(),
                                            })
                                    })
                                    .collect();

                                if let Some(first_function_call) = function_calls.first() {
                                    return Ok((
                                        LlmMessage::assistant_function_run(
                                            first_function_call.clone(),
                                            new_agent_id(),
                                        ),
                                        raw_output
                                            .usage
                                            .map(|u| RequestUsage {
                                                prompt_tokens: u.prompt_tokens as i32,
                                                completion_tokens: u.completion_tokens as i32,
                                            })
                                            .unwrap_or(RequestUsage {
                                                prompt_tokens: 0,
                                                completion_tokens: 0,
                                            }),
                                    ));
                                }
                            } else {
                                // Handle missing tool_calls (important!)
                                return Err(anyhow::anyhow!(
                                    "FinishReason is ToolCalls, but no tool_calls found"
                                ));
                            }
                            None // This is unreachable, but needed for type checking
                        } else if let Some(content) = &message.content {
                            //content handling
                            return Ok((
                                LlmMessage::assistant_text(content.clone(), new_agent_id()),
                                raw_output
                                    .usage
                                    .map(|u| RequestUsage {
                                        prompt_tokens: u.prompt_tokens as i32,
                                        completion_tokens: u.completion_tokens as i32,
                                    })
                                    .unwrap_or(RequestUsage {
                                        prompt_tokens: 0,
                                        completion_tokens: 0,
                                    }),
                            ));
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(llm_message) = llm_message {
                    return Ok((
                        llm_message,
                        raw_output
                            .usage
                            .map(|u| RequestUsage {
                                prompt_tokens: u.prompt_tokens as i32,
                                completion_tokens: u.completion_tokens as i32,
                            })
                            .unwrap_or(RequestUsage {
                                prompt_tokens: 0,
                                completion_tokens: 0,
                            }),
                    ));
                }
            }

            Err(anyhow::anyhow!("Could not convert to LlmMessage"))
        }
        Err(_e) => Err(anyhow::anyhow!("Failed to get reply from OpenAI: {:?}", _e)),
    }
}

pub async fn run_test() {
    dotenv::dotenv().ok();
    let functions = serde_json::json!([
        {
            "name": "get_current_weather",
            "description": "Get the current weather in a given location",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                    },
                    "unit": {
                        "type": "string",
                        "enum": ["celsius", "fahrenheit"]
                    }
                },
                "required": ["location"]
            }
        }
    ]);

    // let system_prompt = r#"you're a tool use "#;

    let input = "";

    let res = chat_wrapper_openai(
        &OPENAI_CONFIG,
        "you're tool use assistant",
        "tell me a popular joke in the 1960s",
        300,
    )
    .await
    .unwrap();

    println!("msg: {:?} \n usage: {:?} ", res.0, res.1.completion_tokens);
}
