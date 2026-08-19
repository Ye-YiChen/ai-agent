use std::{fs, path::Path};

use async_openai::types::chat::{
    ChatCompletionRequestMessageContentPartImageArgs, ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContentPart::{self}, CreateChatCompletionRequestArgs, ImageUrl,
};
use base64::Engine;

pub async fn analyze_image(file_path: &str, query: &str, model: &str) -> anyhow::Result<String> {
    let path = Path::new(file_path);
    let bytes = fs::read(path)?;

    let mime = match path.extension().and_then(|ext| ext.to_str()) {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    };

    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let data_url = format!("data:{mime};base64,{encoded}");

    let message = ChatCompletionRequestUserMessageArgs::default()
        .content(vec![
            ChatCompletionRequestUserMessageContentPart::Text(
                ChatCompletionRequestMessageContentPartTextArgs::default()
                    .text(query)
                    .build()?,
            ),
            ChatCompletionRequestUserMessageContentPart::ImageUrl(
                ChatCompletionRequestMessageContentPartImageArgs::default()
                    .image_url(ImageUrl {
                        url: data_url,
                        detail: None,
                    })
                    .build()?,
            ),
        ])
        .build()?;

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(vec![message.into()])
        .max_tokens(1024u32)
        .build()?;

    let client = async_openai::Client::new();
    let response = client.chat().create(request).await?;

    response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| anyhow::anyhow!("No content in vision response"))
}
