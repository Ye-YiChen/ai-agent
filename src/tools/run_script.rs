use serde_json::{Value, json};
use tokio::process::Command;

use crate::{agent::ExecutionContext, tools::tool::Tool};

/// run_script 工具：执行任意 shell 命令/脚本，通常配合技能(Skill)使用
/// （例如运行技能目录下的 python/node 脚本）。
///
/// 安全说明：本工具能执行任意命令(RCE)，防护完全依赖 ApprovalCallback 的
/// 人工确认——每次执行前都会把完整命令展示给用户、并高亮潜在危险片段，
/// 由用户 y/n 决定是否放行。（这是演示用途，非严肃生产环境。）
pub struct RunScriptTool;

#[async_trait::async_trait]
impl Tool for RunScriptTool {
    fn name(&self) -> &str {
        "run_script"
    }

    fn description(&self) -> &str {
        "执行一条 shell 命令或脚本，通常用于运行技能(Skill)目录下的脚本，\
例如 'python3 skills/xxx/run.py --arg val'。执行前会请求用户确认。"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "要执行的完整命令行，例如 'python3 skills/xxx/run.py'"
                },
                "working_dir": {
                    "type": "string",
                    "description": "可选，命令的工作目录，默认为当前目录"
                }
            },
            "required": ["command"]
        })
    }

    fn source(&self) -> &str {
        "Skills"
    }

    async fn execute(&self, args_json: &str, _context: &ExecutionContext) -> anyhow::Result<String> {
        let args: Value = serde_json::from_str(args_json)?;
        let command = args
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("缺少 command 参数"))?;
        let working_dir = args.get("working_dir").and_then(Value::as_str);

        // 通用执行：交给 sh -c，支持任意 python/node/bash 命令。
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("命令启动失败：{e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);

        Ok(format!(
            "exit_code: {code}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            stdout.trim_end(),
            stderr.trim_end()
        ))
    }
}
