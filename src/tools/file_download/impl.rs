use serde_json::{Value, json};

use crate::{
    agent::ExecutionContext,
    tools::{file_download::execute::download, tool::Tool},
};

/// download_file 工具：从一个 URL 下载文件保存到本地（file 功能包的一部分）。
/// 典型用途：配合 skill-installer 从网络下载 SKILL.md / 资源到 skills/ 目录。
///
/// 安全：内置 SSRF 防护——请求前解析目标 host，拦截 localhost / 环回 /
/// 私有网段 / 链路本地地址，以及额外的 9./11./21./30. 段。
pub struct DownloadFileTool;

#[async_trait::async_trait]
impl Tool for DownloadFileTool {
    fn name(&self) -> &str {
        "download_file"
    }

    fn description(&self) -> &str {
        "从给定 URL 下载文件保存到本地。参数：url(必须是可直接下载的直链)、\
dest_path(可选，保存路径；不填则用 URL 文件名存到当前目录)。出于安全会拦截内网地址。"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "文件下载直链(http/https)"
                },
                "dest_path": {
                    "type": "string",
                    "description": "可选，保存到的本地路径，例如 skills/weather/SKILL.md"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, args_json: &str, _context: &ExecutionContext) -> anyhow::Result<String> {
        let args: Value = serde_json::from_str(args_json)?;
        let url_str = args
            .get("url")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("缺少 url 参数"))?;
        let dest_path = args.get("dest_path").and_then(Value::as_str);

        download(url_str, dest_path).await
    }
}
