// 全功能交互式 AI Agent —— 直接 `cargo run` 启动
//
// 集成了当前项目的所有功能点：
//   - 多轮对话记忆（复用同一个 ExecutionContext）
//   - Agent 自主多步循环（对话走 DeepSeek）
//   - 工具箱 build_full_toolbox：
//       · calculator          精确计算
//       · web_search          Tavily 联网搜索（需 TAVILY_API_KEY）
//       · unzip/list/read     文件解压、目录浏览、文本读取
//       · read_image          图片解读（VISION_MODEL，走 OpenRouter）
//       · delete_file         删除文件（受审批保护）
//       · Expense MCP         费用记录的增删查改（子进程）
//   - ApprovalCallback：delete_file 等高危操作执行前人工确认
//   - SearchCompressorCallback：web_search 超长结果自动向量压缩（embedding 走 OpenRouter）
//   - termimad：把模型返回的 Markdown 渲染成带样式的终端输出
//
// 交互命令：/help 帮助  /reset 清空记忆  /tokens 查看用量  /model 切换模型  /raw 切换Markdown渲染  /exit 退出
use std::io::Write;
use std::sync::Arc;

use ai_agent::{
    agent::{Agent, ContentItem, ExecutionContext, ToolProgress},
    callback::{approval::ApprovalCallback, search_compressor::SearchCompressorCallback},
    constant::{DEEPSEEK_FLASH, DEEPSEEK_V4_PRO, VISION_MODEL},
    tools::{ToolBox, build_full_toolbox},
};
use chrono::Local;
use termimad::MadSkin;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // 交互体验优先：默认只打印 WARN 及以上，避免工具日志刷屏。
    // 想看详细执行过程可设环境变量 RUST_LOG=info。
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("正在初始化工具箱（启动 Expense MCP 子进程，请稍候）...");
    // 加载技能（skills/ 目录），用于注入索引 + 构造 use_skill 工具
    let skills = Arc::new(ai_agent::skill::load_skills("skills"));
    if !skills.is_empty() {
        println!("已加载 {} 个技能", skills.len());
    }
    let toolbox = Arc::new(build_full_toolbox(VISION_MODEL, skills.clone()).await?);

    // 按来源（MCP / 内置）对工具分组，用于启动展示（toolbox 本身要交给 Agent 持有）
    let tool_groups = group_tools_by_source(&toolbox);

    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let instructions = build_instructions(&now, DEEPSEEK_FLASH, &skills);

    let agent = Agent::new(DEEPSEEK_FLASH, Some(&instructions), toolbox)
        .with_max_steps(15)
        // delete_file / run_script 属高危操作，执行前需人工确认
        .with_before_tool_callback(Arc::new(ApprovalCallback::new(["delete_file", "run_script"])))
        // web_search 结果过长时自动向量压缩，节省 token
        .with_after_tool_callback(Arc::new(SearchCompressorCallback));
    let mut agent = agent;

    print_banner(&tool_groups);
    println!("当前模型：{}（用 /model 可切换 flash/pro）", agent.model());

    // Markdown 终端渲染皮肤；render_markdown 控制是否启用（/raw 可切换）
    let skin = MadSkin::default();
    let mut render_markdown = true;

    // 整个会话共用一个 context，实现多轮记忆
    let mut context = ExecutionContext::new();

    loop {
        let line = read_line_async("\n你 > ").await?;
        let input = line.trim();

        if input.is_empty() {
            continue;
        }

        match input {
            "/exit" | "/quit" => {
                println!("再见！");
                break;
            }
            "/reset" => {
                context = ExecutionContext::new();
                println!("(已清空对话记忆)");
                continue;
            }
            "/tokens" => {
                let u = &context.usage;
                println!(
                    "(累计 token 用量：prompt={} completion={} total={})",
                    u.prompt_tokens, u.completion_tokens, u.total_tokens
                );
                continue;
            }
            "/help" => {
                print_help(&tool_groups);
                continue;
            }
            "/raw" => {
                render_markdown = !render_markdown;
                println!(
                    "(Markdown 渲染已{})",
                    if render_markdown { "开启" } else { "关闭" }
                );
                continue;
            }
            cmd if cmd.starts_with("/model") => {
                match cmd.strip_prefix("/model").unwrap_or("").trim() {
                    "" => println!(
                        "(当前模型：{}；用 /model flash 或 /model pro 切换)",
                        agent.model()
                    ),
                    "flash" => {
                        agent.set_model(DEEPSEEK_FLASH);
                        agent.set_instructions(build_instructions(&now, DEEPSEEK_FLASH, &skills));
                        println!("(已切换到 flash：{DEEPSEEK_FLASH})");
                    }
                    "pro" => {
                        agent.set_model(DEEPSEEK_V4_PRO);
                        agent.set_instructions(build_instructions(&now, DEEPSEEK_V4_PRO, &skills));
                        println!("(已切换到 pro：{DEEPSEEK_V4_PRO})");
                    }
                    other => println!("(未知模型 '{other}'，可用：flash / pro)"),
                }
                continue;
            }
            _ => {}
        }

        // 记录调用前的事件数，之后据此统计"本轮"新产生的工具调用
        let events_before = context.events.len();

        print!("\nAI > ");
        std::io::stdout().flush().ok();

        // 流式输出：每个 token 到达即打印，形成打字机效果
        let stream_result = agent
            .chat_stream(
                &mut context,
                input,
                |delta| {
                    print!("{delta}");
                    std::io::stdout().flush().ok();
                },
                |progress| match progress {
                    ToolProgress::Start(names) => {
                        println!("\n  ⚙ 正在调用工具：{} ...", names.join("、"));
                    }
                    ToolProgress::Done(names) => {
                        println!("  ✓ 工具完成：{}，继续思考...\n", names.join("、"));
                        print!("AI > ");
                        std::io::stdout().flush().ok();
                    }
                },
            )
            .await;
        println!();

        match stream_result {
            Ok(output) => {
                // 流式过程打印的是裸文本；若开启 Markdown 渲染，结束后再重绘一遍带样式的版本
                if render_markdown && !output.trim().is_empty() {
                    println!("---");
                    skin.print_text(&output);
                }
                println!(
                    "  ↳ 本会话累计 token：{}（输入 /tokens 查看明细）",
                    context.usage.total_tokens
                );
                let tools_used = collect_tool_usage(&context, events_before);
                if tools_used.is_empty() {
                    println!("  ↳ 本次未调用工具");
                } else {
                    let summary = tools_used
                        .iter()
                        .map(|(name, count)| format!("{name}×{count}"))
                        .collect::<Vec<_>>()
                        .join("、");
                    println!("  ↳ 本次调用工具：{summary}");
                }
            }
            Err(err) => {
                eprintln!("\n[出错] {err}");
                eprintln!("  ↳ 可继续提问，或输入 /reset 重开、/exit 退出。");
            }
        }
    }

    Ok(())
}

/// 组装系统提示词：告诉模型它的身份、当前模型、有哪些能力、如何使用工具。
fn build_instructions(current_time: &str, model: &str, skills: &[ai_agent::skill::Skill]) -> String {
    let base = format!(
        r#"你是一位专业、可靠、乐于帮助用户的全能 AI 助手，基于 DeepSeek 大模型构建，
当前运行的模型是 `{model}`。当用户问你是什么模型时，如实回答你基于 DeepSeek，当前模型为 `{model}`。

当前本地时间：{current_time}
请把"今天""昨天""本周""上个月"等相对时间，都理解为相对于上面的当前时间。

你可以调用以下工具来完成任务：
1. calculator：需要精确计算（金额、百分比、倍数等）时使用，不要心算。
2. web_search：需要最新信息（新闻、天气、汇率、股票、赛事、任何时效性内容）时使用。
3. 文件类工具（unzip、list_files、read_file、read_image、delete_file）：
   处理本地压缩包/文件时，先解压、再列目录、按需读取文本或图片确认内容后再下结论；
   delete_file 会删除文件，属于高危操作，会在执行前请求用户确认。
4. Expense MCP 工具（如 create_expense、list_expenses、get_summary 等）：
   查询、统计、新增、修改、删除费用记录时使用。

原则：
- 能直接回答就直接回答，不必为了用工具而用工具。
- 需要外部数据时必须调用工具获取，不要凭空编造数据，也不要回答"我不知道"。
- 工具返回结果后，直接据此给出自然、简洁、准确的回答，不要向用户复述工具调用过程。"#
    );

    // 追加技能索引（渐进式披露：这里只列 name+description，正文按需用 use_skill 加载）
    format!("{base}{}", ai_agent::skill::build_skill_index(skills))
}

fn print_banner(groups: &[(String, Vec<String>)]) {
    let total: usize = groups.iter().map(|(_, t)| t.len()).sum();
    println!("\n============================================");
    println!("  全功能 AI Agent 已就绪 (DeepSeek + 多工具)");
    println!("============================================");
    println!("已加载 {total} 个工具，按来源分组：");
    print_tool_groups(groups);
    println!("输入问题开始对话；输入 /help 查看命令。");
}

fn print_help(groups: &[(String, Vec<String>)]) {
    println!("\n可用命令：");
    println!("  /help    显示本帮助");
    println!("  /reset   清空对话记忆，重开一段会话");
    println!("  /tokens  查看本会话累计 token 用量");
    println!("  /model   查看/切换模型（/model flash | /model pro）");
    println!("  /raw     切换 Markdown 渲染 / 原始文本显示");
    println!("  /exit    退出（Ctrl-D 同样可退出）");
    println!("\n已加载工具（按来源分组）：");
    print_tool_groups(groups);
    println!("示例问题：");
    println!("  - 帮我算一下 1999 * 3 + 500 等于多少");
    println!("  - 搜一下最近的 AI 大新闻");
    println!("  - 我七月在 Food 分类上花了多少钱？");
    println!("  - 看看 github/github-mcp-server 最近的几个 issue");
}

/// 统一的分组打印：每个来源一行，列出其工具。
fn print_tool_groups(groups: &[(String, Vec<String>)]) {
    for (source, names) in groups {
        println!("  【{}】({}) {}", source, names.len(), names.join(", "));
    }
}

/// 按工具的 `source()` 把工具箱分组，返回 (来源, 工具名列表)。
/// 分组顺序固定：内置工具在前，其余 MCP 来源按名称排序在后；组内工具名排序。
fn group_tools_by_source(toolbox: &ToolBox) -> Vec<(String, Vec<String>)> {
    use std::collections::BTreeMap;

    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for tool in toolbox.values() {
        map.entry(tool.source().to_string())
            .or_default()
            .push(tool.name().to_string());
    }

    let mut groups: Vec<(String, Vec<String>)> = map.into_iter().collect();
    // 组内按工具名排序
    for (_, names) in groups.iter_mut() {
        names.sort();
    }
    // "内置工具" 永远排在最前面
    groups.sort_by(|a, b| match (a.0 == "内置工具", b.0 == "内置工具") {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.0.cmp(&b.0),
    });
    groups
}

/// 统计从 `events_before` 之后新增事件里各工具被调用的次数，按首次出现顺序返回。
fn collect_tool_usage(context: &ExecutionContext, events_before: usize) -> Vec<(String, u32)> {
    let mut usage: Vec<(String, u32)> = Vec::new();
    for event in context.events.iter().skip(events_before) {
        for item in &event.content {
            if let ContentItem::ToolCall { name, .. } = item {
                if let Some(entry) = usage.iter_mut().find(|(n, _)| n == name) {
                    entry.1 += 1;
                } else {
                    usage.push((name.clone(), 1));
                }
            }
        }
    }
    usage
}

/// 在阻塞线程里读取一行输入，避免阻塞 tokio 运行时。
/// 遇到 EOF（Ctrl-D）时返回 "/exit" 以便优雅退出。
async fn read_line_async(prompt: &str) -> anyhow::Result<String> {
    let prompt = prompt.to_string();
    let line = tokio::task::spawn_blocking(move || {
        print!("{prompt}");
        std::io::stdout().flush().ok();
        let mut buf = String::new();
        let n = std::io::stdin().read_line(&mut buf)?;
        Ok::<_, std::io::Error>(if n == 0 { "/exit".to_string() } else { buf })
    })
    .await??;
    Ok(line)
}
