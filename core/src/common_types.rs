use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::{collections::HashMap, io::Result};

use crate::{
    agent_runtime::{AgentId, TopicId},
    tool::FunctionCallInput,
};

static STORE: Lazy<HashMap<String, Func>> = Lazy::new(|| HashMap::new());

pub type Func = Box<dyn Fn(&[u8]) -> Result<String> + Send + Sync>;

pub enum ChatMessage {
    TextMessage(TextMessage),
    MultiModalMessage(MultiModalMessage),
    ToolCallMessage(ToolCallMessage),
    ToolCallResultMessage(ToolCallResultMessage),
    StopMessage(String),
}

pub trait GetContent {
    fn get_content(&self) -> Cow<'_, str>;
}

impl GetContent for ChatMessage {
    fn get_content(&self) -> Cow<'_, str> {
        match self {
            ChatMessage::TextMessage(msg) => Cow::Borrowed(&msg.content),
            ChatMessage::MultiModalMessage(msg) => msg.content.get_content(),
            ChatMessage::ToolCallMessage(msg) => Cow::Owned(format!("{:?}", msg.content.content)),
            ChatMessage::ToolCallResultMessage(msg) => {
                Cow::Owned(format!("{:?}", msg.content.content))
            }
            ChatMessage::StopMessage(content) => Cow::Borrowed(content),
        }
    }
}

impl GetContent for MultiModalContent {
    fn get_content(&self) -> Cow<'_, str> {
        Cow::Borrowed("MultiModal Content")
    }
}

impl GetContent for TextMessage {
    fn get_content(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.content)
    }
}

impl GetContent for ToolCallContent {
    fn get_content(&self) -> Cow<'_, str> {
        Cow::Owned(format!("{:?}", self.content))
    }
}

impl GetContent for ToolCallResultContent {
    fn get_content(&self) -> Cow<'_, str> {
        Cow::Owned(format!("{:?}", self.content))
    }
}

impl GetContent for TextContent {
    fn get_content(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.text)
    }
}

pub struct TextMessage {
    pub content: String,
    pub source: AgentId,
}

pub struct MultiModalMessage {
    pub content: MultiModalContent,
    pub source: AgentId,
}

pub struct ToolCallMessage {
    pub content: ToolCallContent,
    pub source: AgentId,
}

#[derive(Debug)]
pub struct ToolCallResultMessage {
    pub content: ToolCallResultContent,
    pub source: AgentId,
}

pub struct ToolCallContent {
    pub content: Vec<FunctionCallInput>,
}

#[derive(Debug)]
pub struct ToolCallResultContent {
    pub content: Vec<FunctionExecutionResult>,
}

pub struct ChatMessageTrack {
    pub history: Vec<ChatMessage>,
    pub summary: String,
    pub compression_rules: Func,
}

pub struct TextContent {
    pub text: String,
}

pub struct ImageContent {
    pub image: &'static [u8],
}

pub enum AssistantMessageContent {
    FunctionCallInput(FunctionCallInput),
    TextContent(TextContent),
}

pub struct Image {
    pub image: &'static [u8],
}

pub enum MultiModalContent {
    Text(String),
    Image(Image),
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
    pub content: AssistantMessageContent,
    pub source: AgentId,
}

pub struct FunctionExecutionResultMessage {
    pub content: FunctionExecutionResult,
    pub source: AgentId,
}

#[derive(Debug)]
pub struct FunctionExecutionResult {
    pub content: String,
    pub call_id: String,
}

impl LlmMessage {
    pub fn system(content: impl Into<String>, source: impl Into<AgentId>) -> Self {
        LlmMessage::SystemMessage(SystemMessage {
            content: TextContent {
                text: content.into(),
            },
            source: source.into(),
        })
    }

    pub fn user_text(content: impl Into<String>, source: impl Into<AgentId>) -> Self {
        LlmMessage::UserMessage(UserMessage {
            content: MultiModalContent::Text(content.into()),
            source: source.into(),
        })
    }

    pub fn user_image(image_data: &'static [u8], source: impl Into<AgentId>) -> Self {
        LlmMessage::UserMessage(UserMessage {
            content: MultiModalContent::Image(Image { image: image_data }),
            source: source.into(),
        })
    }

    pub fn assistant_text(content: impl Into<String>, source: impl Into<AgentId>) -> Self {
        LlmMessage::AssistantMessage(AssistantMessage {
            content: AssistantMessageContent::TextContent(TextContent {
                text: content.into(),
            }),
            source: source.into(),
        })
    }

    pub fn assistant_function_call(
        function_call: FunctionCallInput,
        source: impl Into<AgentId>,
    ) -> Self {
        LlmMessage::AssistantMessage(AssistantMessage {
            content: AssistantMessageContent::FunctionCallInput(function_call),
            source: source.into(),
        })
    }

    pub fn function_result(
        content: impl Into<String>,
        call_id: impl Into<String>,
        source: impl Into<AgentId>,
    ) -> Self {
        LlmMessage::FunctionExecutionResultMessage(FunctionExecutionResultMessage {
            content: FunctionExecutionResult {
                content: content.into(),
                call_id: call_id.into(),
            },
            source: source.into(),
        })
    }
}

pub struct ChatMessageContext {
    pub sender: AgentId,
    pub topic_id: TopicId,
    pub is_rpc: bool,
}
