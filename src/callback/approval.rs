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

        // 扫描参数里的潜在危险片段并高亮，帮助用户在审批时快速判断
        let dangers = scan_dangers(&tool_call.arguments.to_string());
        if !dangers.is_empty() {
            println!("🚨 检测到潜在危险片段: {}", dangers.join("、"));
        }

        let approved = tokio::task::spawn_blocking(|| {
            print!("是否执行？(y/N): ");
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

/// 常见的高危命令/操作片段，命中即在审批时高亮提示（仅提示，不阻断）。
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf", "rm -r", "sudo", "mkfs", "dd if=", "> /dev/", ":(){",
    "chmod 777", "chmod -r", "curl", "wget", "shutdown", "reboot",
    "eval", "> /etc", "/dev/sd", "kill -9", "killall", "mv /", "> /",
];

/// 扫描文本里命中的危险片段（大小写不敏感），返回命中的模式列表。
fn scan_dangers(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    DANGEROUS_PATTERNS
        .iter()
        .filter(|p| lower.contains(&p.to_lowercase()))
        .copied()
        .collect()
}
