use serde::{Deserialize, Serialize};

use crate::msg_types::{
    AgentId, ContentData, Func, FunctionExecutionResult, GetContent, ImageContent,
    MultiModalContent, TextContent
};
use crate::tool_types::FunctionCallInput;

#[derive(Clone)]
pub enum ChatMessage {
    TextMessage(TextMessage),
    MultiModalMessage(MultiModalMessage),
    ToolCallMessage(ToolCallMessage),
    ToolCallResultMessage(ToolCallResultMessage),
    StopMessage(String),
}

impl GetContent for ChatMessage {
    fn get_content(&self) -> ContentData<'_> {
        match self {
            ChatMessage::TextMessage(msg) => ContentData::Text( msg.content.text.to_string()),
            ChatMessage::MultiModalMessage(msg) => match &msg.content {
                MultiModalContent::Text(TextContent { text: tex }) => {
                    ContentData::Text(tex.clone())
                }
                MultiModalContent::Image(ImageContent { image: img }) => ContentData::Image(img),
            },
            ChatMessage::ToolCallMessage(msg) => {
                ContentData::Text(format!("{:?}", msg.content.content))
            }
            ChatMessage::ToolCallResultMessage(msg) => {
                ContentData::Text(format!("{:?}", msg.content.content))
            }
            ChatMessage::StopMessage(content) => ContentData::Text(content.clone()),
        }
    }
}
#[derive(Clone)]
pub struct TextMessage {
    pub content: TextContent,
    pub source: AgentId,
}
#[derive(Clone)]
pub struct MultiModalMessage {
    pub content: MultiModalContent,
    pub source: AgentId,
}
#[derive(Clone)]
pub struct ToolCallMessage {
    pub content: ToolCallContent,
    pub source: AgentId,
}

#[derive(Debug,Clone)]
pub struct ToolCallResultMessage {
    pub content: ToolCallResultContent,
    pub source: AgentId,
}

#[derive(Clone)]
pub struct ToolCallContent {
    pub content: Vec<FunctionCallInput>,
}

#[derive(Debug, Clone)]
pub struct ToolCallResultContent {
    pub content: Vec<FunctionExecutionResult>,
}

pub struct ChatMessageTrack {
    pub history: Vec<ChatMessage>,
    pub summary: String,
    pub compression_rules: Func,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AssistantMessageContent {
    FunctionCallInput(FunctionCallInput),
    TextContent(TextContent),
}
