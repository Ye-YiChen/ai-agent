use anyhow::Ok;
use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};

pub async fn chat_complete(
    mode: &str, // 模型名称，例如 "deepseek-v4-flash"
    system: Option<&str>, // 可选的系统消息，用于设置对话的上下文
    prompt: &str, // 用户的输入消息
) -> anyhow::Result<String> {
    let client = async_openai::Client::new();
    let mut messages = vec![];

    // 这里是将系统消息添加到消息列表中，如果提供了系统消息的话ß
    if let Some(system) = system {
        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                // .role(async_openai::types::Role::System)
                .content(system)
                .build()?
                .into(),
        );
    }
    // 这里是将用户消息添加到消息列表中
    messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            // .role(async_openai::types::Role::User)
            .content(prompt)
            .build()?
            .into(),
    );

    // 创建一个聊天完成请求，并设置模型、消息和最大令牌数
    let request = CreateChatCompletionRequestArgs::default()
        .model(mode)
        .messages(messages)
        .max_tokens(2048u32)
        .build()?;
    let response = client.chat().create(request).await?;

    tracing::info!("response: {:?}", response);

    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    Ok(content)
}
