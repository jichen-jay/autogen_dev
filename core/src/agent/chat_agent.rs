use crate::msg_types::chat_msg_types::{
    MultiModalMessage, TextMessage, ToolCallResultContent, ToolCallResultMessage,
};
use crate::msg_types::{
    chat_msg_types::ChatMessage, llm_msg_types::LlmMessage, ChatMessageContext, CodeBlock,
    CodeResult, FinishReason, MultiModalContent, RequestUsage, ResponseFormat, TextContent,
};
use crate::msg_types::{AgentId, FunctionExecutionResult, ImageContent, TopicId};
use crate::tool_types::{FunctionCallInput, Tool};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

pub static STORE: Lazy<Mutex<HashMap<String, Tool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub struct LlmCompletionContext {
    pub messages: Vec<LlmMessage>,
    pub state: HashMap<String, Vec<LlmMessage>>,
}

impl LlmCompletionContext {
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
        todo!()
    }
    pub fn load_state(self, state: HashMap<String, LlmMessage>) {
        todo!()
    }
}

pub struct Agent {
    pub name: String,
    pub description: String,
    pub chat_context: Vec<ChatMessage>,
}

pub struct ToolUseAgent {
    pub agent_base: Agent,
    pub llm_context: LlmCompletionContext,
    pub tool_schema: Vec<Value>,
    pub registered_tools: Vec<Tool>,
}

pub struct LlmCompletionAgent {
    pub agent_base: Agent,
    pub llm_context: LlmCompletionContext,
    pub model_client: LlmCompletionClient,
    pub system_messages: Vec<LlmMessage>,
    pub tool_schema: Vec<Value>,
    pub registered_tools: Vec<Tool>,
}

pub struct CodeExecAgent {
    pub agent_base: Agent,
    pub llm_context: LlmCompletionContext,
    pub code_exec_engine: Engine,
}

pub struct Engine;

impl Agent {
    pub async fn on_message(&mut self, message: ChatMessage) -> ChatMessage {
        todo!()
    }

    pub async fn custom_base_agent_logic(in_agent: AgentId, input: &str) -> (AgentId, &str) {
        todo!()
    }

    pub fn registered_tools(&self) -> Vec<Tool> {
        todo!()
    }
}

impl ToolUseAgent {
    pub async fn on_message(&mut self, message: ChatMessage) -> ChatMessage {
        todo!()
    }

    pub async fn custom_base_agent_logic(in_agent: AgentId, input: &str) -> (AgentId, &str) {
        todo!()
    }

    pub fn registered_tools(&self) -> Vec<Tool> {
        todo!()
    }
}

impl CodeExecAgent {
    pub fn execute_code_blocks(&self, code_blocks: Vec<CodeBlock>) -> CodeResult {
        todo!()
    }
}

impl LlmCompletionAgent {
    async fn on_message(&mut self, message: ChatMessage, ctx: ChatMessageContext) {
        let msg: LlmMessage = match message {
            ChatMessage::TextMessage(tex) => {
                LlmMessage::user_text(tex.content.text, ctx.sender)
            }
            ChatMessage::MultiModalMessage(mm) => match mm.content {
                MultiModalContent::Text(tex) => {
                    LlmMessage::user_text(tex.text, ctx.sender)
                }
                MultiModalContent::Image(img) => {
                    LlmMessage::user_image(img.image.into(), ctx.sender)
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
                let call_id = TopicId::new(Some("placeholder"));
                let source = AgentId::new(Some("placehlder"));
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
                LlmMessage::user_text(text, ctx.sender)
            }
            ChatMessage::StopMessage(stp) => LlmMessage::user_text(stp, ctx.sender),
        };
        self.llm_context.add_message(msg).await;
    }

    async fn on_reset(&mut self, message: Reset, ctx: ChatMessageContext) {
        self.llm_context.clear().await;
    }

    async fn on_response_now(
        &mut self,
        message: ResponseNow,
        ctx: ChatMessageContext,
    ) -> ChatMessage {
        let response = self.generate_response(message.response_format, ctx).await;

        response
    }

    async fn on_publish_now(
        &mut self,
        message: PublishNow,
        ctx: ChatMessageContext,
    ) -> ChatMessage {
        let response = self.generate_response(message.response_format, ctx).await;

        response
    }

    async fn generate_response(
        &mut self,
        response_format: ResponseFormat,
        ctx: ChatMessageContext,
    ) -> ChatMessage {
        let mut messages = self.system_messages.clone();
        messages.extend(self.llm_context.clone().get_message().await.into_iter());

        let response = self
            .model_client
            .clone()
            .create(
                messages,
                self.registered_tools.clone(),
                response_format == ResponseFormat::JsonObject,
                HashMap::new(),
            )
            .await;

        match response.content {
            ResultContent::TextContent(tc) => {
                let msg = LlmMessage::assistant_text(tc.text.clone(), AgentId::new(Some("source")));
                self.llm_context.add_message(msg).await;

                ChatMessage::TextMessage(TextMessage {
                    content: tc,
                    source: ctx.sender,
                })
            }
            ResultContent::FunctionCallContent(fcc) => {
                let func_name = fcc.function_name;
                let arguments_w_val = fcc.arguments_obj;

                let binding = STORE.lock().unwrap();
                let func = binding.get(&func_name);
                let raw_result: String = func.expect("failed to get tool").run(arguments_w_val).expect("failed run");

                let tcrm: ToolCallResultContent = ToolCallResultContent {
                    content: vec![FunctionExecutionResult {
                        content: raw_result,
                        call_id: TopicId::new(Some("place")),
                    }],
                };

                ChatMessage::ToolCallResultMessage(ToolCallResultMessage {
                    content: tcrm,
                    source: ctx.sender,
                })
            }
            ResultContent::MultiModalContent(mmc) => {
                match mmc {
                    MultiModalContent::Text(tc) => ChatMessage::TextMessage(TextMessage {
                        content: tc,
                        source: ctx.sender,
                    }),

                    MultiModalContent::Image(ic) => {
                        ChatMessage::MultiModalMessage(MultiModalMessage {
                            content: MultiModalContent::Image(ImageContent { image: ic.image }),
                            source: ctx.sender,
                        })
                    }
                };
                todo!()
            }
        }
    }
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
pub struct LlmCompletionClient {
    messages: Vec<LlmMessage>,
    tools: Vec<Tool>,
    json_output: bool,
    extra_create_args: HashMap<String, Value>,
}

impl LlmCompletionClient {
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
