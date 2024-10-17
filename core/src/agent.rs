use crate::common_types::*;
use crate::tool::*;
use serde_json::Value;
use std::collections::HashMap;

pub struct Agent {
    pub id: String,
    pub meta_data: String,
}

impl Agent {
    async fn on_messages(self) {}

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
    pub registered_tools: Option<Vec<FunctionTool>>,
}

pub struct Schema;

pub struct Engine;

impl BaseChatAgent {
    async fn on_messages(self, messages: ChatMessage) -> ChatMessage {
        todo!()
    }
}

pub trait ToolUse<BaseChatAgent> {
    fn registered_tools(self) -> Vec<FunctionTool>;
}

pub trait CodeExecute<BaseChatAgent> {
    //need to add the engine for code execution
    fn execute_code_blocks(self, code_blocks: Vec<CodeBlock>) -> CodeResult;
}

pub struct ChatCompletionContext;

impl ChatCompletionContext {
    pub async fn add_message(self, message: LlmMessage) {
        todo!()
    }
    pub async fn get_message(self) -> LlmMessage {
        todo!()
    }
    pub async fn clear(self) {
        todo!()
    }

    pub fn save_state(self) -> HashMap<String, LlmMessage> {
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
pub struct CreateResult {
    pub finish_reason: FinishReason,
    pub content: TextContent,
    pub usage: RequestUsage,
}

pub struct ModelCapabilities {
    pub function_calling: bool,
    pub json_output: bool,
    pub vision: bool,
}

pub struct ChatCompletionClient {
    messages: Vec<LlmMessage>,
    tools: Vec<FunctionTool>,
    json_output: bool,
    extra_create_args: HashMap<String, Value>,
}

impl ChatCompletionClient {
    pub async fn create(
        self,
        messages: Vec<LlmMessage>,
        tools: Vec<FunctionTool>,
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

    pub fn count_tokens(self, messages: Vec<LlmMessage>, tools: Vec<FunctionTool>) -> i16 {
        todo!()
    }

    pub fn remainning_tokens(self, messages: Vec<LlmMessage>, tools: Vec<FunctionTool>) -> i16 {
        todo!()
    }
}

pub struct ChatCompletionAgent {
    pub description: String,
    pub model_client: ChatCompletionClient,
    pub system_message: Vec<ChatMessage>,
    pub model_context: ChatCompletionContext,
    // pub code_exec_engine: Option<Engine>,
    pub tools: Vec<FunctionTool>,
}

impl ChatCompletionAgent {
    pub async fn on_text_message(self, message: ChatMessage, ctx: ChatMessageContext) {
        let msg: LlmMessage = LlmMessage::user_text(message.get_content(), ctx.sender);
        self.model_context.add_message(msg).await;
    }

    
}
