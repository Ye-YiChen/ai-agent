use anyhow::Ok;
use async_openai::types::chat::{
    ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionTools, CreateChatCompletionRequest,
    CreateChatCompletionRequestArgs,
};

use crate::tools::{ToolBox, calculator::execute::{CalculatorArgs, calculator}};

pub async fn chat_complete(
    mode: &str,                      // 模型名称，例如 "deepseek-v4-flash"
    system: Option<&str>,            // 可选的系统消息，用于设置对话的上下文
    prompt: &str,                    // 用户的输入消息
    tools: &ToolBox, // 可选的工具列表，用于扩展聊天功能
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
        .messages(messages.clone())
        .tools(tools.clone())
        .max_tokens(2048u32)
        .build()?;
    let response = client.chat().create(request).await?;

    tracing::info!("response: {:?}", response);

    let message = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No message in response"))?
        .message;

    // 看看是否有工具调用的字段
    if let Some(tool_calls) = message.tool_calls {
        message.push(
            ChatCompletionRequestAssistantMessageArgs::default()
                .tool_calls(tool_calls.clone())
                .build()?
                .into(),
        );

        for tool_call in tool_calls {
            match tool_call {
                ChatCompletionMessageToolCalls::Function(function_call) => {
                    let function_name = function_call.name;
                    let function_arguments = function_call.arguments;
                    tracing::info!(
                        "Function call: {} with arguments: {:?}",
                        function_name,
                        function_arguments
                    );

                    if function_name == "calculator" {
                        let args: CalculatorArgs = serde_json::from_str(&function_arguments)?;
                        let result =
                            calculator(&args.operator, args.first_number, args.second_number)?;

                        let tool_result = match result {
                            Ok(value) => value.to_string(),
                            Err(e) => format!("Error: {}", e),
                        };

                        tracing::info!("Calculator result: {}", tool_result);
                        message.push(
                            ChatCompletionRequestToolMessageArgs::default()
                                .tool_call_id(function_call.id.clone())
                                .content(tool_result)
                                .build()?
                                .into(),
                        );
                    }
                }
                _ => {
                    tracing::warn!("Unsupported tool call: {:?}", tool_call);
                }
            }
        }

        // 重新发送消息以获取最终的响应
        let request = CreateChatCompletionRequestArgs::default()
            .model(mode)
            .messages(messages.clone())
            .tools(tools.clone())
            .max_tokens(2048u32)
            .build()?;
        let response = client.chat().create(request).await?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| anyhow::anyhow!("No content in response after tool call"))?;
        return Ok(content);
    }

    let content = message
        .content
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    Ok(content)
}
