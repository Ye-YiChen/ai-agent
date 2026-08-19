use std::{collections::HashMap, sync::Arc};

use crate::tools::{
    calculator::r#impl::CalculatorTool, file_delete::r#impl::DeleteFileTool, file_list::r#impl::ListFileTool, file_read::r#impl::ReadFileTool, file_unzip::r#impl::UnzipFileTool, mcp::{client::McpClient, tool::McpTool}, read_image::r#impl::ReadImageTool, tool::Tool, web_search::r#impl::WebSearchTool,
};

pub mod calculator;
pub mod mcp;
pub mod tool;
pub mod web_search;

pub mod file_unzip;
pub mod file_list;
pub mod file_read;
pub mod file_delete;

pub mod read_image;

pub type ToolBox = HashMap<String, Box<dyn Tool>>;

pub async fn build_toolbox() -> anyhow::Result<ToolBox> {
    let mut tools: Vec<Box<dyn Tool>> = vec![Box::new(CalculatorTool), Box::new(WebSearchTool)];

    // 这里是连接 MCP Server 并获取工具列表，然后把它们包装成 McpTool 并加入到工具箱中
    let mcp_client = Arc::new(McpClient::connect().await?);
    for tool in mcp_client.list_tools().await? {
        tools.push(Box::new(McpTool::new(mcp_client.clone(), tool)));
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