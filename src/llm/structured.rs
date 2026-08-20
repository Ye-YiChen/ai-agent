/* 格式化输出的complete */
use anyhow::Ok;
use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, 
    CreateChatCompletionRequestArgs, ResponseFormatJsonSchema, ResponseFormat,
};

use crate::models::action_plan::ActionPlan;


pub async fn chat_complete_structured(
    mode: &str, // 模型名称，例如 "deepseek-v4-flash"
    system: Option<&str>, // 可选的系统消息，用于设置对话的上下文
    prompt: &str, // 用户的输入消息
) -> anyhow::Result<ActionPlan> {
    let client = crate::llm::client::deepseek_client()?;
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

    let schema =  schemars::schema_for!(ActionPlan);
    let schema_json = schema.as_value().clone();
    let format_setting = ResponseFormat::JsonSchema{
        json_schema: ResponseFormatJsonSchema{
            description: Some("The response should be a JSON object that conforms to the ActionPlan schema.".to_string()),
            schema: schema_json,
            name: "action_plan".to_string(),
            strict: Some(true)
        },
    };

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
