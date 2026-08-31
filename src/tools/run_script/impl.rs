use serde_json::{Value, json};

use crate::{
    agent::ExecutionContext,
    tools::{run_script::execute::run_script, tool::Tool},
};

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

        run_script(command, working_dir).await
    }
}
