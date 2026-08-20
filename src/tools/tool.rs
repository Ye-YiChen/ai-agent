use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use serde_json::Value;

use crate::agent::ExecutionContext;

#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    
    fn description(&self) -> &str;

    fn parameters(&self) -> Value;

    /// 工具来源分组标签，用于展示时按来源归类。
    /// 本地内置工具用默认值；MCP 工具各自返回对应 server 名。
    fn source(&self) -> &str {
        "内置工具"
    }

    async fn execute(&self, args_json: &str, context: &ExecutionContext) -> anyhow::Result<String>;

    fn definition(&self) -> anyhow::Result<ChatCompletionTools> {
        let function = FunctionObjectArgs::default()
            .name(self.name())
            .description(self.description())
            .parameters(self.parameters())
            .build()
            .map_err(|e| {
                anyhow::anyhow!("Failed to build tool definition for {}: {e}", self.name())
            })?;

        Ok(ChatCompletionTools::Function(ChatCompletionTool {
            function,
        }))
    }
}