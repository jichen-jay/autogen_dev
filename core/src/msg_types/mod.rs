pub mod chat_msg_types;
pub mod llm_msg_types;

use std::error::Error;
use uuid::Uuid;

pub type AgentId = Uuid;
pub type TopicId = Uuid;
pub type SubscriptionId = Uuid;

pub type Func = Box<dyn Fn(&[u8]) -> Result<String, Box<dyn Error>> + Send + Sync>;

#[derive(Clone, Debug)]
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

pub enum MultiModalContent {
    Text(TextContent),
    Image(ImageContent),
}

#[derive(Debug)]
pub struct FunctionExecutionResult {
    pub content: String,
    pub call_id: String,
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

pub struct ChatMessageContext {
    pub sender: AgentId,
    pub topic_id: TopicId,
    pub is_rpc: bool,
}
