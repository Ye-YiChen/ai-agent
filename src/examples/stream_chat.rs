use ai_agent::llm::stream::chat_stream_with_retry;
use ai_agent::{
    constant::DEEPSEEK_FLASH,
    llm::{complete::chat_complete, stream::chat_stream, structured::chat_complete_structured, semaphore::get_semaphore},
};
use anyhow::Ok;
use tokio::task::JoinSet;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let prompts = vec![
        "Hello, how are you?",
        "Can you tell me a joke?",
        "What's the weather like today?",
    ];

    let mut set = JoinSet::new();
    for prompt in prompts {
        let span = tracing::info_span!("chat", prompt = prompt);
        set.spawn(
            async move {
                let permit = get_semaphore().acquire().await?;
                let output = chat_stream_with_retry(
                    DEEPSEEK_FLASH,
                    Some("You are a helpful assistant."),
                    prompt,
                )
                .await?;
                drop(permit); // 释放信号量
                Ok::<_, anyhow::Error>((prompt, output)) 
            }
            .instrument(span),
        )
    }

    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok((prompt, output))) => {
                tracing::info!("Prompt: {prompt}, Output: {output}");
            }
            Ok(Err(err)) => {
                tracing::error!("Error in task: {:?}", err);
            }
            Err(join_err) => {
                tracing::error!("Task join error: {:?}", join_err);
            }
        }
    }
    Ok(())
}
