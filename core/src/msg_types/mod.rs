pub mod chat_msg_types;
pub mod llm_msg_types;

use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

use crate::tool_types::SupportedType;

// pub type Func = Box<dyn Fn(&[u8]) -> Result<String, Box<dyn Error>> + Send + Sync>;

use base64::{
    alphabet::STANDARD,
    engine::{general_purpose::NO_PAD, GeneralPurpose},
    Engine,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(String);

impl AgentId {
    pub fn new(text: Option<&str>) -> Self {
        match text {
            Some(tex) => {
                let encoded = GeneralPurpose::new(&STANDARD, NO_PAD).encode(tex);
                let padded = encoded[..36].to_string();
                AgentId(padded)
            }
            None => AgentId(Uuid::new_v4().to_string()),
        }
    }

    pub fn get_text(&self) -> Option<String> {
        GeneralPurpose::new(&STANDARD, NO_PAD)
            .decode(self.0.as_bytes())
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }
}

pub type SubscriptionId = Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicId(String);

impl TopicId {
    pub fn new(text: Option<&str>) -> Self {
        match text {
            Some(tex) => {
                let encoded = GeneralPurpose::new(&STANDARD, NO_PAD).encode(tex);
                let padded = encoded[..36].to_string();
                TopicId(padded)
            }
            None => TopicId(Uuid::new_v4().to_string()),
        }
    }
    pub fn get_text(&self) -> Option<String> {
        GeneralPurpose::new(&STANDARD, NO_PAD)
            .decode(self.0.as_bytes())
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }
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

pub trait CloneableFn:
    Fn(&[SupportedType]) -> Result<String, Box<dyn Error + Send + Sync>> + Send + Sync
{
    fn clone_box(&self) -> Box<dyn CloneableFn>;
}

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

impl Clone for Box<dyn CloneableFn> {
    fn clone(&self) -> Box<dyn CloneableFn> {
        self.clone_box()
    }
}

pub type Func = Box<dyn CloneableFn>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
}

impl<T: ToString> From<T> for TextContent {
    fn from(text: T) -> Self {
        TextContent {
            text: text.to_string(),
        }
    }
}

impl From<&'static [u8]> for ImageContent {
    fn from(image: &'static [u8]) -> Self {
        ImageContent { image }
    }
}

#[derive(Clone, Debug)]
pub struct ImageContent {
    pub image: &'static [u8],
}

impl GetContent for TextContent {
    fn get_content(&self) -> ContentData<'_> {
        ContentData::Text(self.text.clone())
    }
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

impl GetContent for MultiModalContent {
    fn get_content(&self) -> ContentData<'_> {
        match self {
            MultiModalContent::Text(t) => t.get_content(),
            MultiModalContent::Image(i) => i.get_content(),
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
    pub call_id: TopicId,
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
    pub sender: AgentId,
    pub topic_id: TopicId,
    pub is_rpc: bool,
}
