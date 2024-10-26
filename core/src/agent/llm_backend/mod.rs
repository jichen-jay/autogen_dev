use serde::{Deserialize, Serialize};

pub mod llama;
pub mod openai;
pub mod vision_llama;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum AgentCapability {
    Text,
    Vision,
    Audio,
    ImageGeneration,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct LlmConfig {
    pub model: &'static str,
    pub base_url: &'static str,
    pub context_size: usize,
    pub api_key_str: &'static str,
    pub capabilities: AgentCapability,
}

const TOGETHER_CONFIG: LlmConfig = LlmConfig {
    model: "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo",
    context_size: 8192,
    base_url: "https://api.together.xyz/v1/chat/completions",
    api_key_str: "TOGETHER_API_KEY",
    capabilities: AgentCapability::Text,
};

const TOGETHER_VISION_CONFIG: LlmConfig = LlmConfig {
    model: "meta-llama/Llama-3.2-90B-Vision-Instruct-Turbo",
    context_size: 16000,
    base_url: "https://api.together.xyz/v1/chat/completions",
    api_key_str: "TOGETHER_API_KEY",
    capabilities: AgentCapability::Vision,
};

const CODELLAMA_CONFIG: LlmConfig = LlmConfig {
    model: "codellama/CodeLlama-34b-Instruct-hf",
    context_size: 8192,
    base_url: "https://api.together.xyz/v1/chat/completions",
    api_key_str: "TOGETHER_API_KEY",
    capabilities: AgentCapability::Text,
};

const QWEN_CONFIG: LlmConfig = LlmConfig {
    model: "Qwen/Qwen2-72B-Instruct",
    context_size: 32000,
    base_url: "https://api.deepinfra.com/v1/openai/chat/completions",
    api_key_str: "DEEPINFRA_API_KEY",
    capabilities: AgentCapability::Text,
};

const DEEPSEEK_CONFIG: LlmConfig = LlmConfig {
    model: "deepseek-coder",
    context_size: 16000,
    base_url: "https://api.deepseek.com/chat/completions",
    api_key_str: "SEEK_API_KEY",
    capabilities: AgentCapability::Text,
};

const OPENAI_CONFIG: LlmConfig = LlmConfig {
    model: "gpt-3.5-turbo",
    context_size: 16000,
    base_url: "https://api.openai.com/v1/chat/completions",
    api_key_str: "OPENAI_API_KEY",
    capabilities: AgentCapability::Text,
};
