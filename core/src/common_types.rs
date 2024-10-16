use once_cell::sync::Lazy;
use std::{collections::HashMap, io::Result};

use crate::{
    agent_runtime::{AgentId, TopicId},
    tool::FunctionCallInput,
};

static STORE: Lazy<HashMap<String, Func>> = Lazy::new(|| HashMap::new());

pub type Func = Box<dyn Fn(&[u8]) -> Result<String> + Send + Sync>;

pub enum Message {
    TextMessage(TextContent),
    MultiModalMessage(MultiModalContent),
    FunctionCallMessage(FunctionCallContent),
}

pub struct MessageTrack {
    pub history: Vec<Message>,
    pub summary: String,
    pub compression_rules: Func,
}

pub struct TextContent(pub String);

pub struct ImageContent(pub &'static [u8]);

pub enum MultiModalContent {
    TextContent( String),
    ImageContent( &'static [u8]),
}

pub struct FunctionCallContent(FunctionCallInput);

pub struct TextMessage {
    pub content: TextContent,
}

pub struct MultiModalMessage {
    pub content: MultiModalContent,
}

pub struct FunctionCallMessage {
    pub content: FunctionCallContent,
}

pub enum ResponseFormat {
    Text,
    JsonObject,
}
pub enum FinishReason {
    Stop,
    Length,
    FunctionCall,
    ContentFilter,
}

pub struct RequestUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
}

pub struct CodeBlock {
    pub code: String,
    pub language: String,
}

pub struct CodeResult {
    pub exit_code: u8,
    pub output: String,
}

pub enum LlmMessage {
  SystemMessage(SystemMessage),
  UserMessage(UserMessage),
  AssistantMessage(AssistantMessage),
  FunctionExecutionResultMessage(FunctionExecutionResultMessage),
}

pub struct SystemMessage {
    pub content: TextContent,
    pub source: AgentId,
}

pub struct UserMessage {
    pub content: MultiModalContent,
    pub source: AgentId,
}

pub struct AssistantMessage {
    pub content: MultiModalContent,
    pub source: AgentId,
}

pub struct FunctionExecutionResultMessage {
    pub content: FunctionExecutionResult,
    pub source: AgentId,
}

pub struct FunctionExecutionResult {
    pub content: String,
    pub call_id: String,
}

pub struct MessageContext {
    pub sender: AgentId,
    pub topic_id: TopicId,
    pub is_rpc: bool,
}
