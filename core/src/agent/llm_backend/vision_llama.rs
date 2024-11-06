use std::{collections::HashMap, fs, path::Path};

use crate::agent::llm_backend::{LlmConfig, TOGETHER_VISION_CONFIG};
use crate::msg_types::{llm_msg_types::LlmMessage, RequestUsage, AgentId};
use base64::engine::{general_purpose, Engine as _};
use dotenv::dotenv;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    ClientBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChoiceExt {
    pub index: u32,
    pub message: MessageExt,
    pub finish_reason: Option<FinishReasonExt>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageExt {
    pub role: String,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateChatCompletionResponseExt {
    pub id: Option<String>,
    pub object: String,
    pub created: Option<u64>,
    pub model: String,
    pub choices: Vec<ChoiceExt>,
    pub usage: Option<UsageExt>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UsageExt {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContentItem {
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
    #[serde(rename = "text")]
    Text { text: String },
}

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
    dotenv().ok();

    let mut headers = HeaderMap::new();
    let api_key = std::env::var(&llm_config.api_key_str)?;
    let bearer_token = format!("Bearer {}", api_key);

    let contains_llama = llm_config.model.to_ascii_lowercase().contains("llama");
    assert!(contains_llama, "Model is not a Llama model");

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

    let body_json = json!({
        "model": llm_config.model, // Added this line
        "messages": messages,
        "max_tokens": max_token,
        "temperature": 0.3
    });

    let client = ClientBuilder::new().default_headers(headers).build()?;
    let uri = llm_config.base_url;

    let response = client.post(uri).json(&body_json).send().await?;
    let status = response.status();
    let response_body = response.text().await?;

    if status.is_success() {
        let api_response = serde_json::from_str::<CreateChatCompletionResponseExt>(&response_body)?;
        if let Some(llm_message) = output_llmmessage(api_response.clone()) {
            let usage = api_response
                .usage
                .map(|u| RequestUsage {
                    prompt_tokens: u.prompt_tokens.unwrap_or(0) as i32,
                    completion_tokens: u.completion_tokens.unwrap_or(0) as i32,
                })
                .unwrap_or(RequestUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                });
            Ok((llm_message, usage))
        } else {
            Err(anyhow::anyhow!("Could not convert to LlmMessage"))
        }
    } else {
        let error_response: Value = serde_json::from_str(&response_body)?;
        Err(anyhow::anyhow!("API returned an error: {}", error_response))
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
        // If no XML-like structure is found, treat the content as assistant text
        return Some(LlmMessage::assistant_text(data.clone(), AgentId::new(Some("hold"))));
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

pub async fn load_image_and_encode(path: &str) -> String {
    let image_bytes = fs::read(path).expect("Error reading image file");

    let mime_type = match Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        other => panic!("Unsupported image extension: {}", other),
    };

    let encoded_image = general_purpose::STANDARD.encode(&image_bytes);
    let data_uri = format!("data:{};base64,{}", mime_type, encoded_image);

    println!("Image file read and encoded");
    data_uri
}

pub async fn run_test() -> anyhow::Result<()> {
    let functions = serde_json::json!([]); // Not used in this example

    let path = "/home/jaykchen/projects/autogen_dev/assets/cohort_age.png";

    let data_uri = load_image_and_encode(path).await;

    let system_prompt = r#"You're a tool-using AI"#;

    let message_contents = vec![
        MessageContentItem::Text {
            text: "What is funny about this?".to_string(),
        },
        MessageContentItem::ImageUrl {
            image_url: ImageUrl { url: data_uri },
        },
    ];

    let (llm_message, usage) = chat_wrapper_llama_vision(
        &TOGETHER_VISION_CONFIG,
        &system_prompt,
        message_contents,
        1000,
    )
    .await
    .expect("LLM generation failed");

    println!(
        "LLM Message: {:?}\nUsage: {}",
        llm_message, usage.completion_tokens
    );

    Ok(())
}

pub async fn run_test_text() {
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

    let message_contents = vec![MessageContentItem::Text {
        text: "tell me a joke about dogs".to_string(),
    }];
    let res = chat_wrapper_llama_vision(
        &TOGETHER_VISION_CONFIG,
        "you're tool use assistant",
        message_contents,
        300,
    )
    .await
    .unwrap();

    println!("msg: {:?} \n usage: {:?} ", res.0, res.1.completion_tokens);
}
