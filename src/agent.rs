use crate::common_types::*;
use crate::model_client::*;
use crate::tool::*;

pub struct BaseChatAgent {
    pub name: String,
    pub description: String,
    pub model_client: Option<ChatCompletionClient>,
    pub system_message: Option<String>,
    pub model_context: Option<Vec<Message>>,
    pub code_exec_engine: Option<Engine>,
    pub tool_schema: Option<Vec<Schema>>,
    pub registered_tools: Option<Vec<Tool>>,
}

pub struct Schema;

pub struct Engine;

impl BaseChatAgent {
    async fn on_messages(self, messages: Message) -> Message {
        todo!()
    }
}

pub struct CodeBlock {
    pub code: String,
    pub language: String,
}

pub struct CodeResult {
    pub exit_code: u8,
    pub output: String,
}

pub trait ToolUse<BaseChatAgent> {
    fn registered_tools(self) -> Vec<Tool>;
}

pub trait CodeExecute<BaseChatAgent> {
    //need to add the engine for code execution
    fn execute_code_blocks(self, code_blocks: Vec<CodeBlock>) -> CodeResult;
}

pub enum LlmMessage {
    SystemMessage(Content),
    UserMessage(Content),
    AssistantMessage(Content),

    FunctionExecutionResultMessage(FunctionExecutionResult),
}

pub struct FunctionExecutionResult;
pub struct SystemMessage {
    pub content: Content,
}

pub struct AssistantMessage {
    pub content: Content,
}

pub trait CodeAssist<BaseChatAgent> {
    //    async fn on_messages(self, messages: Vec<Message>) -> Message;
    fn on_messages(
        self,
        messages: Vec<Message>,
    ) -> impl std::future::Future<Output = Message> + Send;
}
