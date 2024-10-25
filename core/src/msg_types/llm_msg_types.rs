use crate::msg_types::{
    chat_msg_types::AssistantMessageContent, ImageContent, MultiModalContent, TextContent, FunctionExecutionResult
};
use crate::{
    msg_types::{AgentId, TopicId},
    tool_types::FunctionCallInput,
};

#[derive(Debug, Clone)]
pub enum LlmMessage {
    SystemMessage(SystemMessage),
    UserMessage(UserMessage),
    AssistantMessage(AssistantMessage),
    FunctionExecutionResultMessage(FunctionExecutionResultMessage),
}

#[derive(Debug, Clone)]
pub struct SystemMessage {
    pub content: TextContent,
    pub source: AgentId,
}
#[derive(Clone, Debug)]
pub struct UserMessage {
    pub content: MultiModalContent,
    pub source: AgentId,
}
#[derive(Debug, Clone)]
pub struct AssistantMessage {
    pub content: AssistantMessageContent,
    pub source: AgentId,
}
#[derive(Debug, Clone)]
pub struct FunctionExecutionResultMessage {
    pub content: Vec<FunctionExecutionResult>,
    pub source: AgentId,
}

impl LlmMessage {
    pub fn system(content: impl Into<String>, source: AgentId) -> Self {
        LlmMessage::SystemMessage(SystemMessage {
            content: TextContent {
                text: content.into(),
            },
            source: source.into(),
        })
    }

    pub fn user_text(content: impl Into<String>, source: AgentId) -> Self {
        LlmMessage::UserMessage(UserMessage {
            content: MultiModalContent::Text(TextContent {
                text: content.into(),
            }),
            source: source.into(),
        })
    }

    pub fn user_image(image_data: &'static [u8], source: AgentId) -> Self {
        LlmMessage::UserMessage(UserMessage {
            content: MultiModalContent::Image(ImageContent { image: image_data }),
            source: source.into(),
        })
    }

    pub fn assistant_text(content: impl Into<String>, source: AgentId) -> Self {
        LlmMessage::AssistantMessage(AssistantMessage {
            content: AssistantMessageContent::TextContent(TextContent {
                text: content.into(),
            }),
            source: source.into(),
        })
    }

    pub fn assistant_function_run(function_call: FunctionCallInput, source: AgentId) -> Self {
        LlmMessage::AssistantMessage(AssistantMessage {
            content: AssistantMessageContent::FunctionCallInput(function_call),
            source: source.into(),
        })
    }

    pub fn function_result(
        content: impl Into<String>,
        call_id: impl Into<String>,
        source: AgentId,
    ) -> Self {
        LlmMessage::FunctionExecutionResultMessage(FunctionExecutionResultMessage {
            content: vec![FunctionExecutionResult {
                content: content.into(),
                call_id: call_id.into(),
            }],
            source: source.into(),
        })
    }
}

pub struct ChatMessageContext {
    pub sender: AgentId,
    pub topic_id: TopicId,
    pub is_rpc: bool,
}
