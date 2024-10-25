use autogen_core::agent::llm_agent::run_test;

#[tokio::main]
async fn main() {
    let _ = run_test().await;
}
