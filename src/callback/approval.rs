use std::{collections::HashSet, io::Write};

use crate::agent::{
    ExecutionContext,
    callback::{BeforeToolCallback, ToolCallView},
};

pub struct ApprovalCallback {
    dangerous_tools: HashSet<String>,
}

impl ApprovalCallback {
    pub fn new(dangerous_tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            dangerous_tools: dangerous_tools.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait::async_trait]
impl BeforeToolCallback for ApprovalCallback {
    async fn call(
        &self,
        _context: &ExecutionContext,
        tool_call: ToolCallView<'_>,
    ) -> Option<String> {
        if !self.dangerous_tools.contains(tool_call.name) {
            return None;
        }

        println!("\n⚠️  即将执行高危操作");
        println!("工具: {}", tool_call.name);
        println!("参数: {}", tool_call.arguments);

        let approved = tokio::task::spawn_blocking(|| {
            print!("是否执行？(y/n): ");
            std::io::stdout().flush().ok();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            input.trim().eq_ignore_ascii_case("y")
        })
        .await
        .unwrap_or(false);

        if approved {
            println!("✅ 已批准，继续执行...\n");
            None
        } else {
            println!("❌ 已拒绝，跳过执行\n");
            Some(format!("User denied execution of {}", tool_call.name))
        }
    }
}
