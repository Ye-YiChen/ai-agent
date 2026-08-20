// 示例：结构化输出（让模型直接吐出符合 schema 的 JSON）
// 步骤：提出一个开放式规划问题 -> 模型按 ActionPlan 结构返回 -> 反序列化为强类型对象
// 用到的功能：
//   - chat_complete_structured：用 response_format=json_schema 约束输出（走 DeepSeek）
//   - ActionPlan + schemars：由 Rust 类型自动生成 JSON Schema
// 注意：DeepSeek 若不支持严格 json_schema，可改用 chat_complete_structured_ds（json_object 模式）
use ai_agent::{constant::DEEPSEEK_FLASH, llm::{structured::chat_complete_structured}};
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

    let plan = chat_complete_structured(
        DEEPSEEK_FLASH,
        Some("你是一个全能的助手"),
        "我要去美加墨世界杯观看比赛，如果安排？",
    )
    .await?;

    println!("Response: {plan:#?}");

    Ok(())
}
