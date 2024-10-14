use crate::common_types::*;
use serde_json::Value;
use std::collections::HashMap;

pub struct CreateResult {
    pub finish_reason: FinishReason,
    pub content: Content,
    pub usage: RequestUsage,
}

pub struct ModelCapabilities {
    vision: bool,
    function_calling: bool,
    json_output: bool,
}


pub struct ChatCompletionClient;

impl ChatCompletionClient {
    pub async fn create(
        self,
        messages: Vec<Message>,
        tools: Option<Vec<Value>>,
        json_output: bool,
        extra_create_args: HashMap<String, Value>,
    ) -> CreateResult {
        todo!()
    }

    pub fn capabilities(self) -> ModelCapabilities {
        todo!()
    }
}
