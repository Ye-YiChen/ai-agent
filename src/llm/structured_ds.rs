/* 格式化输出的complete */
use anyhow::Ok;
use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, 
    CreateChatCompletionRequestArgs, ResponseFormatJsonSchema, ResponseFormat,
};
use schemars::schema_for;

use crate::models::action_plan::ActionPlan;


pub async fn chat_complete_structured_ds(
    mode: &str, // 模型名称，例如 "deepseek-v4-flash"
    prompt: &str, // 用户的输入消息
) -> anyhow::Result<ActionPlan> {
    let client = crate::llm::client::deepseek_client()?;
    let mut messages = vec![];

    // 这里是将系统消息添加到消息列表中
    let system = build_system_prompt();
    messages.push(
        ChatCompletionRequestSystemMessageArgs::default()
            // .role(async_openai::types::Role::System)
            .content(system)
            .build()?
            .into(),
    );
    // 这里是将用户消息添加到消息列表中
    messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            // .role(async_openai::types::Role::User)
            .content(prompt)
            .build()?
            .into(),
    );

    let format_setting = ResponseFormat::JsonObject;

    // 创建一个聊天完成请求，并设置模型、消息和最大令牌数
    let request = CreateChatCompletionRequestArgs::default()
        .model(mode)
        .response_format(format_setting)
        .messages(messages)
        .max_tokens(2048u32)
        .build()?;
    let response = client.chat().create(request).await?;

    tracing::info!("response: {:?}", response);

    let plan = response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| anyhow::anyhow!("No content in response"))
        .and_then(|s|serde_json::from_str(&s).map_err(Into::into))?;

    Ok(plan)
}

fn build_system_prompt() -> String {
    let schema = schema_for!(ActionPlan);
    let schema_str = serde_json::to_string_pretty(&schema).unwrap();
    format!(
        r#"You are a helpful assistant. Please provide your response in the following JSON schema format:
        {schema_str}
        "#
    )
}
    