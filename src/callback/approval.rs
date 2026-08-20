use std::{
    collections::HashSet,
    io::Write,
    sync::Mutex,
};

use crate::agent::{
    ExecutionContext,
    callback::{BeforeToolCallback, ToolCallView},
};

pub struct ApprovalCallback {
    dangerous_tools: HashSet<String>,
    /// 用户选择"本任务内一直允许"的工具名，命中则后续不再询问。
    always_allowed: Mutex<HashSet<String>>,
}

impl ApprovalCallback {
    pub fn new(dangerous_tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            dangerous_tools: dangerous_tools.into_iter().map(Into::into).collect(),
            always_allowed: Mutex::new(HashSet::new()),
        }
    }

    /// 清空"本轮一直允许"的记录。应在每次用户新提问开始时调用，
    /// 使得上一轮选择的 'a'(一直允许) 不会延续到下一轮提问。
    pub fn reset(&self) {
        if let Ok(mut set) = self.always_allowed.lock() {
            set.clear();
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

        // 用户此前对该工具选择过"本任务内一直允许"，则不再询问
        if self
            .always_allowed
            .lock()
            .map(|s| s.contains(tool_call.name))
            .unwrap_or(false)
        {
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

        // 返回：'y'=本次允许，'a'=本任务内该工具一直允许，其它(含回车)=拒绝
        let choice = tokio::task::spawn_blocking(|| {
            print!("是否执行？(y=本次 / a=本任务内不再询问 / N=拒绝): ");
            std::io::stdout().flush().ok();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            input.trim().to_lowercase()
        })
        .await
        .unwrap_or_default();

        match choice.as_str() {
            "a" => {
                if let Ok(mut set) = self.always_allowed.lock() {
                    set.insert(tool_call.name.to_string());
                }
                println!("✅ 已批准，且本任务内不再询问 {}\n", tool_call.name);
                None
            }
            "y" => {
                println!("✅ 已批准，继续执行...\n");
                None
            }
            _ => {
                println!("❌ 已拒绝，跳过执行\n");
                Some(format!("User denied execution of {}", tool_call.name))
            }
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
