use autogen_core::msg_types::llm_msg_types::LlmMessage;
use uuid::Uuid;
use webhook_flows::{create_endpoint, request_handler};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    create_endpoint().await;
}

pub async fn try_some() {
    let sender_id = Uuid::default();

    let msg: LlmMessage = LlmMessage::user_text("create a placeholder message", sender_id);

    match msg {
        LlmMessage::SystemMessage(tex) => {
            println!("msg: {:?}", tex.content.text);
        }
        _ => unreachable!(),
    }
}
