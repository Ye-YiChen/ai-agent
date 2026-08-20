use async_openai::{Client, config::OpenAIConfig};

/// 构建一个指向 DeepSeek 的 OpenAI 兼容客户端。
///
/// 读取环境变量：
/// - `DEEPSEEK_API_KEY`（必填）
/// - `DEEPSEEK_BASE_URL`（可选，默认 `https://api.deepseek.com`）
///
/// 说明：
/// - 聊天类调用（complete / stream / structured / agent / gaia）统一使用它，显式连到 DeepSeek。
/// - embedding / vision 仍走默认的 `Client::new()`，即由全局环境变量
///   `OPENAI_BASE_URL` + `OPENAI_API_KEY`（指向 OpenRouter）决定。
pub fn deepseek_client() -> anyhow::Result<Client<OpenAIConfig>> {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .map_err(|_| anyhow::anyhow!("环境变量 DEEPSEEK_API_KEY 未设置"))?;
    let api_base = std::env::var("DEEPSEEK_BASE_URL")
        .unwrap_or_else(|_| "https://api.deepseek.com".to_string());

    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(api_base);

    Ok(Client::with_config(config))
}
