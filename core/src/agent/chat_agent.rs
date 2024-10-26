use crate::msg_types::chat_msg_types::{
    MultiModalMessage, TextMessage, ToolCallResultContent, ToolCallResultMessage,
};
use crate::msg_types::{
    chat_msg_types::ChatMessage, llm_msg_types::LlmMessage, ChatMessageContext, CodeBlock,
    CodeResult, FinishReason, MultiModalContent, RequestUsage, ResponseFormat, TextContent,
};
use crate::msg_types::{
    new_agent_id, new_topic_id, AgentId, FunctionExecutionResult, ImageContent,
};
use crate::tool_types::{FunctionCallInput, Tool};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use super::agent_runtime::ResponseMessage;

pub static STORE: Lazy<Mutex<HashMap<String, Tool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
pub struct Agent {
    pub id: AgentId,
    pub meta_data: String,
}

impl Agent {
    pub async fn on_messages(
        self,
        message: ChatMessage,
        ctx: ChatMessageContext,
    ) -> ResponseMessage {
        // ResponseMessage {
        //     message: ChatMessage::Text {
        //         content: format!("Response from Agent {}", self.id.0),
        //     },
        //     sender: self.id.clone(),
        //     recepient: None,
        // }
        todo!()
    }

    pub fn save_state(self) {}
    pub fn load_state(self) {}
}
pub struct BaseChatAgent {
    pub name: String,
    pub description: String,
    pub model_client: Option<ChatCompletionClient>,
    pub system_message: Option<String>,
    pub model_context: Option<Vec<ChatMessage>>,
    pub code_exec_engine: Option<Engine>,
    pub tool_schema: Option<Vec<Schema>>,
    pub registered_tools: Option<Vec<Tool>>,
}

pub struct Schema;

pub struct Engine;

impl BaseChatAgent {
    async fn on_messages(self, messages: ChatMessage) -> ChatMessage {
        todo!()
    }
}

pub trait ToolUse<BaseChatAgent> {
    fn registered_tools(self) -> Vec<Tool>;
}

pub trait CodeExecute<BaseChatAgent> {
    //need to add the engine for code execution
    fn execute_code_blocks(self, code_blocks: Vec<CodeBlock>) -> CodeResult;
}

#[derive(Debug, Clone)]
pub struct ChatCompletionContext {
    pub messages: Vec<LlmMessage>,
    pub state: HashMap<String, Vec<LlmMessage>>,
}

impl ChatCompletionContext {
    pub async fn add_message(&mut self, message: LlmMessage) {
        self.messages.push(message);
    }
    pub async fn get_message(self) -> Vec<LlmMessage> {
        self.messages.clone()
    }
    pub async fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn save_state(&mut self) -> HashMap<String, LlmMessage> {
        // self.state.insert("placeholder".to_string(), self.messages.clone());
        todo!()
    }
    pub fn load_state(self, state: HashMap<String, LlmMessage>) {
        todo!()
    }
}
pub trait CodeAssist<BaseChatAgent> {
    //    async fn on_messages(self, messages: Vec<Message>) -> ChatMessage;
    fn on_messages(
        self,
        messages: Vec<ChatMessage>,
    ) -> impl std::future::Future<Output = ChatMessage> + Send;
}

pub enum ResultContent {
    TextContent(TextContent),
    MultiModalContent(MultiModalContent),
    FunctionCallContent(FunctionCallInput),
}

pub struct CreateResult {
    pub finish_reason: FinishReason,
    pub content: ResultContent,
    pub usage: RequestUsage,
}

pub struct ModelCapabilities {
    pub function_calling: bool,
    pub json_output: bool,
    pub vision: bool,
}

#[derive(Clone)]
pub struct ChatCompletionClient {
    messages: Vec<LlmMessage>,
    tools: Vec<Tool>,
    json_output: bool,
    extra_create_args: HashMap<String, Value>,
}

impl ChatCompletionClient {
    pub async fn create(
        self,
        messages: Vec<LlmMessage>,
        tools: Vec<Tool>,
        json_output: bool,
        extra_create_args: HashMap<String, Value>,
    ) -> CreateResult {
        todo!()
    }

    pub fn capabilities(self) -> ModelCapabilities {
        todo!()
    }

    pub fn actual_usage(self) -> RequestUsage {
        todo!()
    }

    pub fn total_usage(self) -> RequestUsage {
        todo!()
    }

    pub fn count_tokens(self, messages: Vec<LlmMessage>, tools: Vec<Tool>) -> i16 {
        todo!()
    }

    pub fn remainning_tokens(self, messages: Vec<LlmMessage>, tools: Vec<Tool>) -> i16 {
        todo!()
    }
}

pub struct ResponseNow {
    pub response_format: ResponseFormat,
}

pub struct PublishNow {
    pub response_format: ResponseFormat,
}

pub struct Reset;

pub struct ChatCompletionAgent {
    pub description: String,
    pub model_client: ChatCompletionClient,
    pub system_message: Vec<LlmMessage>,
    pub model_context: ChatCompletionContext,
    pub tools: Vec<Tool>,
}

impl ChatCompletionAgent {
    pub async fn on_message(&mut self, message: ChatMessage, ctx: ChatMessageContext) {
        let msg: LlmMessage = match message {
            ChatMessage::TextMessage(tex) => {
                LlmMessage::user_text(tex.content.text, ctx.sender.unwrap())
            }
            ChatMessage::MultiModalMessage(mm) => match mm.content {
                MultiModalContent::Text(tex) => {
                    LlmMessage::user_text(tex.text, ctx.sender.unwrap())
                }
                MultiModalContent::Image(img) => {
                    LlmMessage::user_image(img.image.into(), ctx.sender.unwrap())
                }
            },
            ChatMessage::ToolCallMessage(tcm) => {
                let mut res = Vec::<String>::new();
                for fc in tcm.content.content {
                    let func_name = fc.function_name;
                    let arguments_w_val = fc.arguments_obj;

                    let binding = STORE.lock().unwrap();
                    let func = binding.get(&func_name).unwrap();
                    let raw_result: String = func.run(arguments_w_val).expect("failed run");

                    res.push(raw_result);
                }
                let call_id = new_topic_id();
                let source = new_agent_id();
                LlmMessage::function_result(res.join(", "), call_id, source)
            }
            ChatMessage::ToolCallResultMessage(tcrm) => {
                let text = tcrm
                    .content
                    .content
                    .into_iter()
                    .map(|x| x.content.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                LlmMessage::user_text(text, ctx.sender.unwrap())
            }
            ChatMessage::StopMessage(stp) => LlmMessage::user_text(stp, ctx.sender.unwrap()),
        };
        self.model_context.add_message(msg).await;
    }

    pub async fn on_reset(&mut self, message: Reset, ctx: ChatMessageContext) {
        self.model_context.clear().await;
    }

    pub async fn on_response_now(
        &mut self,
        message: ResponseNow,
        ctx: ChatMessageContext,
    ) -> ChatMessage {
        let response = self.generate_response(message.response_format, ctx).await;

        response
    }

    pub async fn on_publish_now(
        &mut self,
        message: PublishNow,
        ctx: ChatMessageContext,
    ) -> ChatMessage {
        let response = self.generate_response(message.response_format, ctx).await;

        response
    }

    pub async fn generate_response(
        &mut self,
        response_format: ResponseFormat,
        ctx: ChatMessageContext,
    ) -> ChatMessage {
        let mut messages = self.system_message.clone();
        messages.extend(self.model_context.clone().get_message().await.into_iter());

        let response = self
            .model_client
            .clone()
            .create(
                messages,
                self.tools.clone(),
                response_format == ResponseFormat::JsonObject,
                HashMap::new(),
            )
            .await;

        match response.content {
            ResultContent::TextContent(tc) => {
                let msg = LlmMessage::assistant_text(tc.text.clone(), Uuid::default());
                self.model_context.add_message(msg).await;

                ChatMessage::TextMessage(TextMessage {
                    content: tc,
                    source: ctx.sender.unwrap(),
                })
            }
            ResultContent::FunctionCallContent(fcc) => {
                let func_name = fcc.function_name;
                let arguments_w_val = fcc.arguments_obj;

                let binding = STORE.lock().unwrap();
                let func = binding.get(&func_name).unwrap();
                let raw_result: String = func.run(arguments_w_val).expect("failed run");

                let tcrm: ToolCallResultContent = ToolCallResultContent {
                    content: vec![FunctionExecutionResult {
                        content: raw_result,
                        call_id: new_agent_id().to_string(),
                    }],
                };

                ChatMessage::ToolCallResultMessage(ToolCallResultMessage {
                    content: tcrm,
                    source: ctx.sender.unwrap(),
                })
            }
            ResultContent::MultiModalContent(mmc) => {
                match mmc {
                    MultiModalContent::Text(tc) => ChatMessage::TextMessage(TextMessage {
                        content: tc,
                        source: ctx.sender.unwrap(),
                    }),

                    MultiModalContent::Image(ic) => {
                        ChatMessage::MultiModalMessage(MultiModalMessage {
                            content: MultiModalContent::Image(ImageContent { image: ic.image }),
                            source: ctx.sender.unwrap(),
                        })
                    }
                };
                todo!()
            }
        }
    }

    pub fn save_state(&mut self) -> HashMap<String, LlmMessage> {
        self.model_context.save_state();
        todo!()
    }

    pub fn load_state(&mut self, state: HashMap<String, LlmMessage>) {
        // self.model_context.load_state(state);

        todo!()
    }
}
