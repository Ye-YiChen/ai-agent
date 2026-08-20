use std::{collections::HashMap, sync::Arc};

use crate::skill::Skill;
use crate::tools::{
    calculator::r#impl::CalculatorTool, file_delete::r#impl::DeleteFileTool, file_download::DownloadFileTool, file_list::r#impl::ListFileTool, file_read::r#impl::ReadFileTool, file_unzip::r#impl::UnzipFileTool, file_write::r#impl::WriteFileTool, mcp::{client::McpClient, tool::McpTool}, read_image::r#impl::ReadImageTool, run_script::RunScriptTool, tool::Tool, use_skill::UseSkillTool, web_search::r#impl::WebSearchTool,
};

pub mod calculator;
pub mod mcp;
pub mod tool;
pub mod web_search;

pub mod file_unzip;
pub mod file_list;
pub mod file_read;
pub mod file_delete;
pub mod file_download;
pub mod file_write;

pub mod read_image;

pub mod use_skill;
pub mod run_script;

pub type ToolBox = HashMap<String, Box<dyn Tool>>;

pub async fn build_toolbox() -> anyhow::Result<ToolBox> {
    let mut tools: Vec<Box<dyn Tool>> = vec![Box::new(CalculatorTool), Box::new(WebSearchTool)];

    // 这里是连接 MCP Server 并获取工具列表，然后把它们包装成 McpTool 并加入到工具箱中
    let mcp_client = Arc::new(McpClient::connect().await?);
    for tool in mcp_client.list_tools().await? {
        tools.push(Box::new(McpTool::new(mcp_client.clone(), tool, "Expense MCP")));
    }

    Ok(into_toolbox(tools))
}

/// 构建"全功能"工具箱：本地工具 + 文件工具 + MCP 工具 + Skill 工具，一次性都装进来。
/// 供 `cargo run` 启动的交互式 Agent 使用。
///
/// 包含：
/// - calculator：精确计算
/// - web_search：Tavily 联网搜索
/// - unzip / list_files / read_file / read_image / delete_file：文件探索
/// - use_skill / run_script：技能加载与脚本执行
/// - Expense MCP / GitHub MCP 暴露的工具（子进程）
pub async fn build_full_toolbox(
    vision_model: impl Into<String>,
    skills: Arc<Vec<Skill>>,
) -> anyhow::Result<ToolBox> {
    let mut tools: Vec<Box<dyn Tool>> = vec![
        Box::new(CalculatorTool),
        Box::new(WebSearchTool),
        Box::new(UnzipFileTool),
        Box::new(ListFileTool),
        Box::new(ReadFileTool),
        Box::new(ReadImageTool::new(vision_model)),
        Box::new(DeleteFileTool),
        Box::new(DownloadFileTool),
        Box::new(WriteFileTool),
    ];

    // Skill 工具：use_skill 用于按需加载技能 SOP；run_script 用于执行技能脚本。
    // 只有存在技能时才挂载这两个工具，避免空技能时徒增工具。
    if !skills.is_empty() {
        tools.push(Box::new(UseSkillTool::new(skills.clone())));
        tools.push(Box::new(RunScriptTool));
    }

    // 拉起 MCP Server（expense_mcp_server）并把它暴露的工具也装进来
    let mcp_client = Arc::new(McpClient::connect().await?);
    for tool in mcp_client.list_tools().await? {
        tools.push(Box::new(McpTool::new(mcp_client.clone(), tool, "Expense MCP")));
    }

    // 可选：接入 GitHub 官方 MCP Server。
    // 仅当设置了 GITHUB_PERSONAL_ACCESS_TOKEN 时启用（token 是必需项）；
    // 二进制路径由 GITHUB_MCP_SERVER_PATH 指定，缺省则从 PATH 找 github-mcp-server。
    // 连接失败不影响其它工具。
    if std::env::var("GITHUB_PERSONAL_ACCESS_TOKEN").is_ok() {
        match McpClient::connect_github().await {
            Ok(client) => {
                let client = Arc::new(client);
                match client.list_tools().await {
                    Ok(gh_tools) => {
                        tracing::info!("已接入 GitHub MCP，加载 {} 个工具", gh_tools.len());
                        for tool in gh_tools {
                            tools.push(Box::new(McpTool::new(client.clone(), tool, "GitHub MCP")));
                        }
                    }
                    Err(e) => tracing::warn!("GitHub MCP list_tools 失败，跳过：{e}"),
                }
            }
            Err(e) => tracing::warn!("连接 GitHub MCP 失败，跳过：{e}"),
        }
    }

    Ok(into_toolbox(tools))
}

pub fn build_file_explorer_toolbox(vision_model: impl Into<String>) -> ToolBox {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(UnzipFileTool),
        Box::new(ListFileTool),
        Box::new(ReadFileTool),
        Box::new(ReadImageTool::new(vision_model)),
        Box::new(DeleteFileTool)
    ];
    into_toolbox(tools)
}

fn into_toolbox(tools: Vec<Box<dyn Tool>>) -> ToolBox {
    tools.into_iter().map(|t| (t.name().to_string(), t)).collect()
}