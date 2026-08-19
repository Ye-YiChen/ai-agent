use serde_json::Value;

use crate::agent::{ExecutionContext, ToolResultStatus};

#[derive(Debug, Clone, Copy)]
pub struct ToolCallView<'a> {
    pub tool_call_id: &'a str,
    pub name: &'a str,
    pub arguments: &'a Value
}

#[async_trait::async_trait]
pub trait BeforeToolCallback: Send + Sync {
    async fn call(&self, context: &ExecutionContext, tool_call: ToolCallView<'_>) -> Option<String>;
}

#[async_trait::async_trait]
pub trait AfterToolCallback: Send + Sync {
    async fn call (
        &self,
        context: &ExecutionContext,
        tool_call_id: &str,
        tool_name: &str,
        status: ToolResultStatus,
        content: &str
    ) -> Option<(ToolResultStatus, String)>;
}