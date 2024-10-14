
use serde_json::Value;

type Fn = fn(&[u8]) -> String;

pub struct Message {
    pub source: String,
    pub target: String,
    pub payload: String,
    pub content: Content,
}

pub struct MessageTrack {
    pub history: Vec<Message>,
    pub summary: String,
    pub compression_rules: Fn,
}

pub enum Content {
    Text(String),
    Image(&'static [u8]),
    ToolCall(Vec<FunctionCall>),
    Stop(String),
}

pub struct FunctionCallInput {
    pub tool_meta: Value,
    pub args: Vec<String>,
    pub tool_call_feed: Value,
    pub arg_type: Vec<String>,
}

pub struct FunctionCall {
    pub id: String,
    pub args: &'static [u8],
    pub name: String,
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
