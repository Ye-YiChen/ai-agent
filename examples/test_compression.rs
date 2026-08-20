// 示例：Agent + 搜索结果自动压缩
// 步骤：Agent 收到问题 -> 调用 web_search -> 结果过长时用向量检索压缩 -> 只保留相关片段再回答
// 用到的功能：
//   - Agent::run：多步自主循环（对话走 DeepSeek，上限 5 步）
//   - SearchCompressorCallback：after_tool 回调，对 web_search 结果做向量压缩（embedding 走 OpenRouter）
use ai_agent::{
    agent::Agent, callback::search_compressor::SearchCompressorCallback, constant::DEEPSEEK_FLASH, tools::build_toolbox,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let toolbox = Arc::new(build_toolbox().await?);

    let agent = Agent::new(
        DEEPSEEK_FLASH,
        Some("你是一个善用网页搜索的助手".to_string()),
        toolbox,
    )
    .with_max_steps(5)
    .with_after_tool_callback(Arc::new(SearchCompressorCallback));

    let result = agent.run("2026世界人工智能大会 WAIC 有什么亮点？").await?;
    println!("\n最终回答: {}", result.output);

    Ok(())
}
