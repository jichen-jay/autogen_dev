use crate::msg_types::{
    chat_msg_types::AssistantMessageContent,
    llm_msg_types::{
        AssistantMessage, FunctionExecutionResultMessage, LlmMessage, SystemMessage, UserMessage,
    },
    FunctionExecutionResult, MultiModalContent, TextContent,
};
use crate::tool_types::FunctionCallInput;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestFunctionMessage, ChatCompletionRequestFunctionMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageArgs,
};

impl From<SystemMessage> for ChatCompletionRequestMessage {
    fn from(msg: SystemMessage) -> Self {
        ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(msg.content.text) // Assuming TextContent has a .text field
                .build()
                .unwrap(), // Consider handling potential errors here
        )
    }
}

impl From<UserMessage> for ChatCompletionRequestMessage {
    fn from(msg: UserMessage) -> Self {
        ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessageArgs::default()
                .content(match msg.content {
                    MultiModalContent::Text(text_content) => text_content.text,
                    MultiModalContent::Image(_) => {
                        todo! {"image content conversion not implemented"}
                    }
                })
                .build()
                .unwrap(),
        )
    }
}

impl From<AssistantMessage> for ChatCompletionRequestMessage {
    fn from(msg: AssistantMessage) -> Self {
        let content = match msg.content {
            AssistantMessageContent::TextContent(text_content) => Some(text_content.text),
            _ => None,
        };

        ChatCompletionRequestMessage::Assistant(
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(content) // Correct conversion
                .build()
                .unwrap(),
        )
    }
}

impl From<LlmMessage> for ChatCompletionRequestMessage {
    fn from(msg: LlmMessage) -> Self {
        match msg {
            LlmMessage::SystemMessage(sys_msg) => sys_msg.into(),
            LlmMessage::UserMessage(user_msg) => user_msg.into(),
            LlmMessage::AssistantMessage(asst_msg) => asst_msg.into(),
            LlmMessage::FunctionExecutionResultMessage(_) => unreachable!(),
        }
    }
}
