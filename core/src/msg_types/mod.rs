pub mod chat_msg_types;
pub mod llm_msg_types;

use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

use crate::tool_types::SupportedType;

pub type AgentId = Uuid;
pub type TopicId = Uuid;
pub type SubscriptionId = Uuid;

// pub type Func = Box<dyn Fn(&[u8]) -> Result<String, Box<dyn Error>> + Send + Sync>;

pub fn new_agent_id() -> AgentId {
    Uuid::new_v4()
}

pub fn new_topic_id() -> TopicId {
    Uuid::new_v4()
}

pub fn new_subscription_id() -> SubscriptionId {
    Uuid::new_v4()
}

#[derive(Debug)]
pub struct FunctionToolError(String);

impl std::fmt::Display for FunctionToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for FunctionToolError {}

// Parsing functions as before...

// Now define the CloneableFn trait without inheriting Clone
pub trait CloneableFn:
    Fn(&[SupportedType]) -> Result<String, Box<dyn Error + Send + Sync>> + Send + Sync
{
    fn clone_box(&self) -> Box<dyn CloneableFn>;
}

// Implement CloneableFn for all suitable types
impl<T> CloneableFn for T
where
    T: Fn(&[SupportedType]) -> Result<String, Box<dyn Error + Send + Sync>>
        + Send
        + Sync
        + Clone
        + 'static,
{
    fn clone_box(&self) -> Box<dyn CloneableFn> {
        Box::new(self.clone())
    }
}

// Implement Clone for Box<dyn CloneableFn>
impl Clone for Box<dyn CloneableFn> {
    fn clone(&self) -> Box<dyn CloneableFn> {
        self.clone_box()
    }
}

// Define your Func type alias
pub type Func = Box<dyn CloneableFn>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
}

impl GetContent for TextContent {
    fn get_content(&self) -> ContentData<'_> {
        ContentData::Text(self.text.clone())
    }
}

#[derive(Clone, Debug)]
pub struct ImageContent {
    pub image: &'static [u8],
}

impl GetContent for ImageContent {
    fn get_content(&self) -> ContentData<'_> {
        ContentData::Image(self.image)
    }
}

#[derive(Clone, Debug)]
pub enum Content {
    Text(TextContent),
    Image(ImageContent),
}

#[derive(Clone, Debug)]
pub enum ContentData<'a> {
    Text(String),
    Image(&'a [u8]),
}

pub trait GetContent {
    fn get_content(&self) -> ContentData<'_>;
}

impl GetContent for Content {
    fn get_content(&self) -> ContentData<'_> {
        match self {
            Content::Text(text) => text.get_content(),
            Content::Image(image) => image.get_content(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum MultiModalContent {
    Text(TextContent),
    Image(ImageContent),
}

#[derive(Debug, Clone)]
pub struct FunctionExecutionResult {
    pub content: String,
    pub call_id: String,
}

#[derive(PartialEq)]
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

pub struct ChatMessageContext {
    pub sender: Option<AgentId>,
    pub topic_id: Option<TopicId>,
    pub is_rpc: bool,
}
