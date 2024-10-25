use std::{collections::HashMap, sync::Arc, time::Duration};

use async_openai::{
    config::{Config, OpenAIConfig},
    types::{
        ChatCompletionRequestArgs, ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionResponse,
        ChatCompletionStreamOptions, CreateChatCompletionRequest, CreateCompletionRequestArgs,
        MessageRole,
    },
    Client as OAI_Client,
};
use reqwest::{Client as HttpClient, Response};
use serde_json::{json, Value};

use crate::{
    msg_types::{
        chat_msg_types::AssistantMessageContent,
        llm_msg_types::{LlmMessage, SystemMessage, UserMessage},
        AgentId, RequestUsage,
    },
    tool_types::Tool,
};

use super::chat_agent::{CreateResult, ModelCapabilities};

pub struct AsyncOpenAI {
    pub client: OAI_Client<dyn Config>,
    pub timeout: Option<Duration>,
    pub max_retries: u8,
    pub default_headers: HashMap<String, String>,
    pub default_query: HashMap<String, Value>,
    pub http_client: HttpClient,
}

impl AsyncOpenAI {
    pub fn new(api_key: &'static str) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = OAI_Client::with_config(config);

        AsyncOpenAI {
            client,
            timeout: Some(Duration::from_secs(30)),
            max_retries: 3,
            default_headers: HashMap::new(),
            default_query: HashMap::new(),
            http_client: HttpClient::new(),
        }
    }
}

pub struct BaseOpenAIChatCompletionClient {
    pub client: Arc<AsyncOpenAI>,
    pub create_args: ChatCompletionRequestArgs<'static>,
    pub model_capabilities: Option<ModelCapabilities>,
    pub total_usage: RequestUsage,
    pub actual_usage: RequestUsage,
}

impl BaseOpenAIChatCompletionClient {
    pub fn new(
        client: Arc<AsyncOpenAI>,
        model_capabilities: Option<ModelCapabilities>,
        model: String,
    ) -> Self {
        let create_args = ChatCompletionRequestArgs::default()
            .model(model)
            .temperature(1.0)
            .max_tokens(256)
            .top_p(1.0)
            .frequency_penalty(0.0)
            .presence_penalty(0.0)
            .n(1)
            .echo(false);

        BaseOpenAIChatCompletionClient {
            client,
            create_args,
            model_capabilities,
            total_usage: RequestUsage::default(),
            actual_usage: RequestUsage::default(),
        }
    }
    pub async fn create(
        &mut self,
        messages: Vec<LlmMessage>,
        tools: Vec<Tool>,
        json_output: bool,
        extra_create_args: HashMap<String, Value>,
    ) -> CreateResult {
        let messages: Vec<async_openai::types::ChatCompletionRequestMessage> = messages
            .iter()
            .map(|message| {
                match message {
                    LlmMessage::UserMessage(UserMessage { content, source: _ }) => {
                        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                            content: Some(content.to_string()),
                            name: None,
                        })
                    }
                    LlmMessage::SystemMessage(SystemMessage { content, source: _ }) => {
                        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                            content: Some(content.to_string()),
                        })
                    }
                    LlmMessage::AssistantMessage(AssistantMessage { content, source: _ }) => {
                        ChatCompletionRequestMessage::Assistant(
                            ChatCompletionRequestAssistantMessage {
                                content: Some(content.to_string()),
                            },
                        )
                    }
                    LlmMessage::FunctionExecutionResultMessage(_) => {
                        // Handle function execution result message appropriately.
                        // You'll likely need to create a custom struct or use a different variant
                        // depending on your implementation and how OpenAI expects this information.
                        todo!()
                    }
                }
            })
            .collect();

        let mut request = CreateChatCompletionRequest::default()
            .messages(messages)
            .clone();

        if !tools.is_empty() {
            let tools_json: Vec<Value> = tools.iter().map(|tool| tool.to_json()).collect();
            // Assume there's a field "tools" in CreateChatCompletionRequest
            request = request.tools(tools_json);
        }

        if json_output {
            // Assuming a response_format field exists in create_args
            request = request.response_format(Some(vec!["json".to_string()]));
        }

        // Update create_args based on extra_create_args
        for (key, value) in extra_create_args {
            match key.as_str() {
                "temperature" => {
                    request = request.temperature(value.as_f64().unwrap_or(1.0) as f32);
                }
                "max_tokens" => {
                    request = request.max_tokens(value.as_u64().unwrap_or(256) as u32);
                }
                "top_p" => {
                    request = request.top_p(value.as_f64().unwrap_or(1.0) as f32);
                }
                "frequency_penalty" => {
                    request = request.frequency_penalty(value.as_f64().unwrap_or(0.0) as f32);
                }
                "presence_penalty" => {
                    request = request.presence_penalty(value.as_f64().unwrap_or(0.0) as f32);
                }
                _ => {}
            }
        }

        let response = self.client.client.chat().create(request).await;

        match response {
            Ok(ChatCompletionResponse {
                choices,
                usage: openai_usage,
                ..
            }) => {
                if let Some(usage) = openai_usage {
                    self.total_usage.merge(RequestUsage {
                        prompt_tokens: usage.prompt_tokens as i32,
                        completion_tokens: usage.completion_tokens as i32,
                    });
                    self.actual_usage.merge(RequestUsage {
                        prompt_tokens: usage.prompt_tokens as i32,
                        completion_tokens: usage.completion_tokens as i32,
                    });
                }

                let message = match choices.first() {
                    Some(choice) => &choice.message,
                    None => {
                        return CreateResult {
                            finish_reason: "stop".into(),
                            content: crate::msg_types::ResultContent::TextContent(
                                crate::msg_types::TextContent {
                                    text: "No response received from OpenAI".into(),
                                },
                            ),
                            usage: self.total_usage.clone(),
                        }
                    }
                };

                CreateResult {
                    finish_reason: "stop".into(),
                    content: crate::msg_types::ResultContent::TextContent(
                        crate::msg_types::TextContent {
                            text: message.content.clone().unwrap_or_default(),
                        },
                    ),
                    usage: self.total_usage.clone(),
                }
            }
            Err(error) => CreateResult {
                finish_reason: "error".into(),
                content: crate::msg_types::ResultContent::TextContent(
                    crate::msg_types::TextContent {
                        text: error.to_string(),
                    },
                ),
                usage: self.total_usage.clone(),
            },
        }
    }
}
