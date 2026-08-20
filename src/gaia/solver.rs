use std::sync::Arc;

use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, FinishReason, ResponseFormat,
};
use backon::{ExponentialBuilder, Retryable};

use crate::{agent::Agent, gaia::models::GaiaOutput, tools::ToolBox};

pub async fn solve_problem_with_retry(
    model: &str,
    system: &str,
    prompt: &str,
) -> anyhow::Result<GaiaOutput> {
    let op = || async { solve_problem(model, system, prompt).await };
    op.retry(ExponentialBuilder::default().with_max_times(3))
        .await
}

async fn solve_problem(model: &str, system: &str, prompt: &str) -> anyhow::Result<GaiaOutput> {
    // DeepSeek 不支持严格 json_schema，改用 json_object（prompt 里需含 "json" 字样）
    let format_setting = ResponseFormat::JsonObject;

    let client = crate::llm::client::deepseek_client()?;
    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(json_prompt(prompt))
                .build()?
                .into(),
        ])
        .response_format(format_setting)
        .build()?;

    let response = client.chat().create(request).await?;

    let choice = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No choices in response"))?;

    if choice.finish_reason == Some(FinishReason::ContentFilter) {
        return Ok(GaiaOutput {
            is_solvable: false,
            unsolvable_reason: "Model refuse to answer".to_string(),
            final_answer: String::new(),
        });
    }

    let content = choice
        .message
        .content
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;
    let output: GaiaOutput = serde_json::from_str(&content)?;

    Ok(output)
}

pub async fn solve_problem_with_tools(
    model: &str,
    system: &str,
    prompt: &str,
    toolbox: Arc<ToolBox>,
) -> anyhow::Result<GaiaOutput> {
    // 用普通 run 而非 run_structured：DeepSeek 思考模式不支持 tool_choice=required
    let agent = Agent::new(model, Some(system), toolbox).with_max_steps(30);
    let result = agent.run(&json_prompt(prompt)).await?;
    parse_gaia_output(&result.output)
}

/// 在 prompt 末尾追加 JSON 输出要求，引导模型返回结构化字段。
/// 同时满足 DeepSeek json_object 模式要求 prompt 中出现 "json" 字样的约定。
fn json_prompt(prompt: &str) -> String {
    format!(
        "{prompt}\n\n请以 JSON 对象格式输出最终结果，字段为：\
is_solvable(布尔)、unsolvable_reason(字符串)、final_answer(字符串)。\
不要输出任何 JSON 以外的内容。"
    )
}

/// 把模型返回文本解析成 GaiaOutput，容忍被 markdown 代码块包裹的情况。
fn parse_gaia_output(text: &str) -> anyhow::Result<GaiaOutput> {
    let mut s = text.trim();
    if let Ok(out) = serde_json::from_str::<GaiaOutput>(s) {
        return Ok(out);
    }
    // 剥离 ```json / ``` 围栏后重试
    if let Some(rest) = s.strip_prefix("```json") {
        s = rest;
    } else if let Some(rest) = s.strip_prefix("```") {
        s = rest;
    }
    if let Some(rest) = s.strip_suffix("```") {
        s = rest;
    }
    serde_json::from_str::<GaiaOutput>(s.trim())
        .map_err(|e| anyhow::anyhow!("解析 GaiaOutput JSON 失败: {e}; 原文: {text}"))
}