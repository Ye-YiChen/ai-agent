use ai_agent::{constant::DEEPSEEK_FLASH, llm::{complete::chat_complete, structured::chat_complete_structured, structured_ds::chat_complete_structured_ds}};
use anyhow::Ok;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let plan = chat_complete_structured_ds(
        DEEPSEEK_FLASH,
        "Hello, how are you?",
    )
    .await?;

    println!("Chat completion result: {plan:#?}");
    Ok(())
}
