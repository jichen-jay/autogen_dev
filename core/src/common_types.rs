use std::{collections::HashMap, io::Result};
use once_cell::sync::Lazy;

use crate::tool::FunctionCallInput;

static STORE: Lazy<HashMap<String, Func>> = Lazy::new(|| HashMap::new());

pub type Func = Box<dyn Fn(&[u8]) -> Result<String> + Send + Sync>;


pub struct Message {
    pub source: String,
    pub target: String,
    pub payload: String,
    pub content: Content,
}

pub struct MessageTrack {
    pub history: Vec<Message>,
    pub summary: String,
    pub compression_rules: Func,
}

pub enum Content {
    Text(String),
    Image(&'static [u8]),
    ToolCallInput(Vec<FunctionCallInput>),
    Stop(String),
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
    SystemMessage(Content),
    UserMessage(Content),
    AssistantMessage(Content),
    FunctionExecutionResultMessage(FunctionExecutionResult),
}

pub struct FunctionExecutionResult {
    pub content: String,
    pub call_id: String,
}

pub struct FunctionExecutionResultMessage {
    pub content: Vec<FunctionExecutionResult>,
}


pub struct SystemMessage {
    pub content: Content,
}

pub struct AssistantMessage {
    pub content: Content,
}

pub struct UserMessage {
    pub content: Content,
}
