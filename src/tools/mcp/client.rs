use anyhow::Result;
use rmcp::model::{CallToolRequestParams, Tool};
use rmcp::service::{RoleClient, RunningService};
use rmcp::{
    ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

/// 对一个 MCP Server 连接的封装。内部启动了一个子进程
/// （expense_mcp_server），通过 stdio 跟它通信。
pub struct McpClient {
    service: RunningService<RoleClient, ()>,
}

impl McpClient {
    /// 通用的 stdio 连接：把任意命令当作 MCP Server 子进程拉起来。
    ///
    /// - `program` / `args`：启动命令及参数
    /// - `envs`：注入给子进程的环境变量（比如各类 API Token）
    ///
    /// 传输方式仍是 stdio —— 由本进程 spawn 子进程，并通过它的
    /// 标准输入输出通信；子进程生命周期随本 client。
    pub async fn connect_stdio(
        program: &str,
        args: &[&str],
        envs: &[(&str, &str)],
    ) -> Result<Self> {
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let envs: Vec<(String, String)> = envs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let service = ()
            .serve(TokioChildProcess::new(Command::new(program).configure(
                move |cmd| {
                    cmd.args(&args);
                    for (k, v) in &envs {
                        cmd.env(k, v);
                    }
                },
            ))?)
            .await?;

        Ok(Self { service })
    }

    /// 把 expense_mcp_server 当子进程拉起来，并建立连接
    pub async fn connect() -> Result<Self> {
        Self::connect_stdio(
            "cargo",
            &["run", "--quiet", "--bin", "expense_mcp_server"],
            &[],
        )
        .await
    }

    /// 连接 GitHub 官方 MCP Server（本地二进制，stdio 模式）。
    ///
    /// 读取环境变量：
    /// - `GITHUB_PERSONAL_ACCESS_TOKEN`：GitHub PAT，用于认证（必填，属于 secret）
    /// - `GITHUB_MCP_SERVER_PATH`：github-mcp-server 二进制路径（可选）。
    ///   支持绝对路径或相对路径（相对于运行 agent 的工作目录）；
    ///   不设置时默认用 `github-mcp-server`，即从系统 PATH 中查找。
    ///
    /// 默认以 `--read-only` 只读模式启动，并启用 context / repos / issues /
    /// pull_requests 四个 toolset。其中 context 提供 `get_me`，让模型能识别
    /// 当前 PAT 对应的用户（"我的仓库/我的 issue" 等需要它）。
    pub async fn connect_github() -> Result<Self> {
        let token = std::env::var("GITHUB_PERSONAL_ACCESS_TOKEN")
            .map_err(|_| anyhow::anyhow!("环境变量 GITHUB_PERSONAL_ACCESS_TOKEN 未设置"))?;
        let path = std::env::var("GITHUB_MCP_SERVER_PATH")
            .unwrap_or_else(|_| "github-mcp-server".to_string());

        Self::connect_stdio(
            &path,
            &[
                "stdio",
                "--read-only",
                "--toolsets",
                "context,repos,issues,pull_requests",
            ],
            &[("GITHUB_PERSONAL_ACCESS_TOKEN", &token)],
        )
        .await
    }

    /// 拿到 server 暴露的所有工具
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let result = self.service.list_tools(Default::default()).await?;
        Ok(result.tools)
    }

    /// 按名字调用某个工具，arguments 是一个 JSON 对象
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<String> {
        let params = CallToolRequestParams::new(name.to_string())
            .with_arguments(arguments.as_object().cloned().unwrap_or_default());

        let result = self.service.call_tool(params).await?;

        let text = result
            .content
            .iter()
            .filter_map(|block| block.as_text().map(|t| t.text.clone()))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }
}
