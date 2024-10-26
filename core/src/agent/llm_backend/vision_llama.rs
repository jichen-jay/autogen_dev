use std::{collections::HashMap, fs::File, io::Read};

use crate::agent::llm_backend::{LlmConfig, TOGETHER_VISION_CONFIG};
use crate::msg_types::{llm_msg_types::LlmMessage, new_agent_id, RequestUsage};
use crate::tool_types::FunctionCallInput;
use base64::engine::{general_purpose, Engine};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    ClientBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Define custom FinishReason to include 'eos'
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FinishReasonExt {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    FunctionCall,
    Eos, // Added for Llama
}

// Define custom Choice structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChoiceExt {
    pub index: u32,
    pub message: MessageExt,
    pub finish_reason: Option<FinishReasonExt>,
}

// Define custom Message structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageExt {
    pub role: String,
    pub content: Option<String>,
}

// Define custom CreateChatCompletionResponse
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateChatCompletionResponseExt {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChoiceExt>,
    pub usage: Option<UsageExt>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContentItem {
    #[serde(rename = "image")]
    Image { data: String }, // Base64-encoded image data
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsageExt {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

// Define the Message struct to hold the role and content
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<MessageContentItem>,
}

pub async fn chat_wrapper_llama_vision(
    llm_config: &LlmConfig,
    system_prompt: &str,
    message_contents: Vec<MessageContentItem>, // Accepts both image and text inputs
    max_token: u16,
) -> anyhow::Result<(LlmMessage, RequestUsage)> {
    let mut headers = HeaderMap::new();
    let api_key = std::env::var(&llm_config.api_key_str)?;
    let bearer_token = format!("Bearer {}", api_key);

    // Ensure that the model is a Llama model
    let contains_llama = llm_config.model.to_ascii_lowercase().contains("llama");
    assert_eq!(contains_llama, true);

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("MyClient/1.0.0"));
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token)?);

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: vec![MessageContentItem::Text {
                text: system_prompt.to_string(),
            }],
        },
        Message {
            role: "user".to_string(),
            content: message_contents,
        },
    ];

    let messages_value = serde_json::to_value(&messages)?;

    let uri = llm_config.base_url;

    let body_json = serde_json::json!({
        "model": llm_config.model,
        "messages": messages_value,
        "max_tokens": max_token,
        "temperature": 0.3
    });

    let body = serde_json::to_vec(&body_json)?;

    let client = ClientBuilder::new().default_headers(headers).build()?;
    match client.post(uri).body(body.clone()).send().await {
        Ok(chat) => {
            let response_body = match chat.text().await {
                Ok(it) => {
                    println!("Response body: {}", it);
                    it
                }

                Err(e) => {
                    println!("error: {:?}", e);
                    panic!()
                }
            };

            let raw_output =
                serde_json::from_str::<CreateChatCompletionResponseExt>(&response_body)?;

            if let Some(llm_message) = output_llmmessage(raw_output.clone()) {
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
                Ok((llm_message, usage))
            } else {
                Err(anyhow::anyhow!("Could not convert to LlmMessage"))
            }
        }
        Err(e) => {
            println!("Error getting response from Llama API: {:?}", e);
            Err(anyhow::anyhow!(
                "Failed to get reply from Llama API: {:?}",
                e
            ))
        }
    }
}

pub async fn chat_wrapper_llama_vision_toolcall(
    llm_config: &LlmConfig,
    functions: &serde_json::Value,
    system_prompt: &str,
    message_contents: Vec<MessageContentItem>, // Accepts both image and text inputs
    max_token: u16,
) -> anyhow::Result<(LlmMessage, RequestUsage)> {
    let mut headers = HeaderMap::new();
    let api_key = std::env::var(&llm_config.api_key_str)?;
    let bearer_token = format!("Bearer {}", api_key);

    // Ensure that the model is a Llama model
    let contains_llama = llm_config.model.to_ascii_lowercase().contains("llama");
    assert_eq!(contains_llama, true);

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("MyClient/1.0.0"));
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token)?);

    // Combine system prompt and functions into the prompt
    let system_prompt_w_tool = format!(
        "{}\n\nHere are the tools you're equipped with: {}\n",
        system_prompt, functions
    );

    let messages = vec![
        Message {
            role: "system".to_string(),
            content: vec![MessageContentItem::Text {
                text: system_prompt_w_tool,
            }],
        },
        Message {
            role: "user".to_string(),
            content: message_contents,
        },
    ];

    let messages_value = serde_json::to_value(&messages)?;

    let uri = llm_config.base_url;

    let body_json = serde_json::json!({
        "model": llm_config.model,
        "messages": messages_value,
        "max_tokens": max_token,
        "temperature": 0.3
    });

    let body = serde_json::to_vec(&body_json)?;

    let client = ClientBuilder::new().default_headers(headers).build()?;
    match client.post(uri).body(body.clone()).send().await {
        Ok(chat) => {
            let response_body = chat.text().await?;
            let raw_output =
                serde_json::from_str::<CreateChatCompletionResponseExt>(&response_body)?;

            if let Some(llm_message) = output_llmmessage(raw_output.clone()) {
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
                Ok((llm_message, usage))
            } else {
                Err(anyhow::anyhow!("Could not convert to LlmMessage"))
            }
        }
        Err(e) => {
            println!("Error getting response from Llama API: {:?}", e);
            Err(anyhow::anyhow!(
                "Failed to get reply from Llama API: {:?}",
                e
            ))
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LlamaToolCall {
    pub name: String,
    pub arguments: Option<HashMap<String, String>>,
}

fn extract_json_from_xml_like(xml_like_data: &str) -> Option<String> {
    let start_tag = "<tool_call>";
    let end_tag = "</tool_call>";

    if let Some(start_pos) = xml_like_data.find(start_tag) {
        if let Some(end_pos) = xml_like_data.find(end_tag) {
            let start = start_pos + start_tag.len();
            let end = end_pos;
            return Some(xml_like_data[start..end].trim().to_string());
        }
    }
    None
}

pub fn output_llmmessage(res_obj: CreateChatCompletionResponseExt) -> Option<LlmMessage> {
    let msg_obj = res_obj.choices[0].message.clone();
    if let Some(data) = msg_obj.content {
        if let Some(json_str) = extract_json_from_xml_like(&data) {
            println!("Extracted JSON string: {:?}", json_str.clone());
            match extract_tool_call(&json_str) {
                Some(tc) => {
                    // Construct FunctionCallInput
                    let function_call_input = FunctionCallInput {
                        function_name: tc.name.clone(),
                        arguments_obj: serde_json::Value::Object(
                            tc.arguments
                                .unwrap_or_default()
                                .into_iter()
                                .map(|(k, v)| (k, serde_json::Value::String(v)))
                                .collect(),
                        ),
                        return_type: "".to_string(),
                    };
                    return Some(LlmMessage::assistant_function_run(
                        function_call_input,
                        new_agent_id(),
                    ));
                }
                None => {
                    // If no tool call is extracted, treat the content as assistant text
                    return Some(LlmMessage::assistant_text(data.clone(), new_agent_id()));
                }
            }
        } else {
            // If no XML-like structure is found, treat the content as assistant text
            return Some(LlmMessage::assistant_text(data.clone(), new_agent_id()));
        }
    }
    None
}

fn extract_tool_call(input: &str) -> Option<LlamaToolCall> {
    // Try to parse the input as JSON directly
    if let Ok(value) = serde_json::from_str::<Value>(input) {
        if let Value::Object(map) = value {
            let name = map.get("name")?.as_str()?.to_string();
            let arguments_value = map.get("arguments")?;
            let arguments_map = match arguments_value {
                Value::Object(args) => args
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_string())))
                    .collect(),
                _ => HashMap::new(),
            };
            return Some(LlamaToolCall {
                name,
                arguments: Some(arguments_map),
            });
        }
    }
    None
}
pub async fn run_test() -> anyhow::Result<()> {
    println!("Starting run_test");

    dotenv::dotenv().ok();
    println!("Environment variables loaded");

    // Define the functions (if any)
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

    println!("Functions defined");

    let system_prompt = r#"You're a tool-using AI"#;

    println!("Reading image file");
    let mut file = match File::open("/home/jaykchen/projects/autogen_dev/assets/tree.jpg") {
        Ok(f) => f,
        Err(e) => {
            println!("Error opening file: {:?}", e);
            return Err(anyhow::anyhow!("Error opening file: {:?}", e));
        }
    };

    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        println!("Error reading file: {:?}", e);
        return Err(anyhow::anyhow!("Error reading file: {:?}", e));
    }

    println!("Image file read successfully, encoding...");

    let encoded_image = general_purpose::STANDARD.encode(&buffer);
    println!("Image file read and encoded");

    let message_contents = vec![
        MessageContentItem::Image {
            data: encoded_image,
        },
        MessageContentItem::Text {
            text: "What is in this image?".to_string(),
        },
    ];

    println!("Message contents prepared");

    // Call the chat_wrapper_llama_vision function
    println!("Calling chat_wrapper_llama_vision");
    let (llm_message, usage) = chat_wrapper_llama_vision(
        &TOGETHER_VISION_CONFIG,
        &system_prompt,
        message_contents,
        1000,
    )
    .await
    .expect("LLM generation failed");
    println!("chat_wrapper_llama_vision returned");

    // Print out the LLM's response and usage information
    println!(
        "LLM Message: {:?}\nUsage: {}",
        llm_message, usage.completion_tokens
    );

    Ok(())
}
