use crate::skill::Skill;

/// 根据技能名加载其 SOP 正文，返回给模型的文本。
/// 找到则返回技能目录 + 操作指南；未找到则返回当前可用技能列表提示。
pub fn use_skill(skills: &[Skill], name: &str) -> String {
    match skills.iter().find(|s| s.name == name) {
        Some(s) => format!(
            "已加载技能「{}」。\n技能目录(base_dir)：{}\n（脚本/资源请相对该目录引用，可用 run_script 执行）\n\n=== 操作指南(SOP) ===\n{}",
            s.name,
            s.base_dir.display(),
            s.body
        ),
        None => {
            let available: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
            format!(
                "未找到技能「{name}」。当前可用技能：{}",
                if available.is_empty() {
                    "(无)".to_string()
                } else {
                    available.join(", ")
                }
            )
        }
    }
}
