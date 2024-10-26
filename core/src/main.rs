use autogen_core::agent::llm_backend::vision_llama::run_test;

#[tokio::main]
async fn main() {
    let _ = run_test().await;
}
