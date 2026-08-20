use std::fs;
use std::path::{Path, PathBuf};

/// 一个 Skill（技能）：预置的专业操作流程。
/// 对应磁盘上 `skills/<name>/SKILL.md` 一个文件夹。
///
/// - `name` / `description` 来自 SKILL.md 的 frontmatter，启动时用于生成
///   "技能索引"注入 system prompt（渐进式披露：平时只暴露这一两行）。
/// - `body` 是 SKILL.md 的正文（SOP），只有被 `use_skill` 加载时才进入上下文。
/// - `base_dir` 是该技能所在目录，SOP 里引用的脚本/资源都相对它。
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub body: String,
    pub base_dir: PathBuf,
}

/// 扫描 skills 根目录，加载其下每个子目录里的 `SKILL.md`。
/// 目录不存在或解析失败都不会报错，只是返回已成功加载的部分（对演示更友好）。
pub fn load_skills(root: impl AsRef<Path>) -> Vec<Skill> {
    let root = root.as_ref();
    let mut skills = Vec::new();

    let Ok(entries) = fs::read_dir(root) else {
        return skills; // skills/ 目录不存在时静默返回空
    };

    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let md = dir.join("SKILL.md");
        if !md.exists() {
            continue;
        }
        match fs::read_to_string(&md) {
            Ok(content) => match parse_skill(&content, &dir) {
                Some(skill) => skills.push(skill),
                None => tracing::warn!("跳过 {}：缺少合法的 frontmatter(name)", md.display()),
            },
            Err(e) => tracing::warn!("读取 {} 失败：{e}", md.display()),
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// 解析 SKILL.md：开头是 `---` 包裹的 YAML frontmatter（取 name / description），
/// 其后为正文。frontmatter 缺少 name 视为非法，返回 None。
fn parse_skill(content: &str, base_dir: &Path) -> Option<Skill> {
    let mut lines = content.lines();

    // 第一行必须是 frontmatter 起始的 "---"
    if lines.next()?.trim() != "---" {
        return None;
    }

    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    let mut front_ended = false;
    let mut body_lines: Vec<&str> = Vec::new();

    for line in lines {
        if !front_ended {
            if line.trim() == "---" {
                front_ended = true;
                continue;
            }
            if let Some(v) = line.strip_prefix("name:") {
                name = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().to_string());
            }
        } else {
            body_lines.push(line);
        }
    }

    if !front_ended {
        return None; // frontmatter 没有正确闭合
    }

    Some(Skill {
        name: name?,
        description: description.unwrap_or_default(),
        body: body_lines.join("\n").trim().to_string(),
        base_dir: base_dir.to_path_buf(),
    })
}

/// 生成注入 system prompt 的"技能索引"文本（只含 name + description）。
/// 无技能时返回空字符串。
pub fn build_skill_index(skills: &[Skill]) -> String {
    if skills.is_empty() {
        return String::new();
    }
    let list = skills
        .iter()
        .map(|s| format!("- {}: {}", s.name, s.description))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"

你还可以使用以下「技能(Skill)」——它们是预置的专业操作流程(SOP)：
{list}

使用方式：当用户的任务匹配某个技能时，先用 use_skill 工具（参数 name）加载它的详细步骤，
再严格按照返回的指南执行；指南中若要求运行脚本，使用 run_script 工具。
没有匹配的技能则正常处理，不要为了用技能而用技能。"#
    )
}
