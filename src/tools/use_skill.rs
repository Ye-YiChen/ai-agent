use std::sync::Arc;

use serde_json::{Value, json};

use crate::{agent::ExecutionContext, skill::Skill, tools::tool::Tool};

/// use_skill 工具：实现技能的"渐进式披露"。
/// 平时 system prompt 里只有各技能的 name+description；模型判断需要某技能时
/// 调用本工具，execute 返回该技能 SKILL.md 的完整正文(SOP)，从而进入上下文。
pub struct UseSkillTool {
    skills: Arc<Vec<Skill>>,
}

impl UseSkillTool {
    pub fn new(skills: Arc<Vec<Skill>>) -> Self {
        Self { skills }
    }
}

#[async_trait::async_trait]
impl Tool for UseSkillTool {
    fn name(&self) -> &str {
        "use_skill"
    }

    fn description(&self) -> &str {
        "加载一个技能(Skill)的详细操作指南(SOP)。当任务匹配某个已列出的技能时，\
先调用本工具获取步骤说明，再据此执行。参数 name 为技能名称。"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "要加载的技能名称，须与已列出的技能名一致"
                }
            },
            "required": ["name"]
        })
    }

    fn source(&self) -> &str {
        "Skills"
    }

    async fn execute(&self, args_json: &str, _context: &ExecutionContext) -> anyhow::Result<String> {
        let args: Value = serde_json::from_str(args_json)?;
        let name = args.get("name").and_then(Value::as_str).unwrap_or_default();

        match self.skills.iter().find(|s| s.name == name) {
            Some(s) => Ok(format!(
                "已加载技能「{}」。\n技能目录(base_dir)：{}\n（脚本/资源请相对该目录引用，可用 run_script 执行）\n\n=== 操作指南(SOP) ===\n{}",
                s.name,
                s.base_dir.display(),
                s.body
            )),
            None => {
                let available: Vec<&str> = self.skills.iter().map(|s| s.name.as_str()).collect();
                Ok(format!(
                    "未找到技能「{name}」。当前可用技能：{}",
                    if available.is_empty() {
                        "(无)".to_string()
                    } else {
                        available.join(", ")
                    }
                ))
            }
        }
    }
}
