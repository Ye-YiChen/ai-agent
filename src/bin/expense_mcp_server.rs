use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData as McpError, ServiceExt, schemars, tool, tool_router, transport::stdio};
use serde::{Deserialize, Serialize};

// 给 expense-tracker-api 这个 web 服务写的一个 MCP Server，
// ---------- 参数 / 请求体类型 ----------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ListExpensesParams {
    /// 按分类筛选，比如 "Food"，大小写不敏感。不填就是不筛选，返回所有分类
    category: Option<String>,
    /// 按月份筛选，格式必须是 "YYYY-MM"。不填就是不筛选月份
    month: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetSummaryParams {
    /// 只统计某一个月，格式 "YYYY-MM"。不填就是统计全部历史数据
    month: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetExpenseParams {
    /// 要查询的费用记录 id，来自 list_expenses 或 create_expense 的返回结果
    id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DeleteExpenseParams {
    /// 要删除的费用记录 id
    id: String,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
struct CreateExpenseParams {
    description: String,
    amount: f64,
    category: String,
    /// ISO 日期格式，"YYYY-MM-DD"
    date: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct UpdateExpenseParams {
    /// 要更新的费用记录 id
    id: String,
    description: Option<String>,
    amount: Option<f64>,
    category: Option<String>,
    /// ISO 日期格式，"YYYY-MM-DD"
    date: Option<String>,
}

/// 这里只把用户真正传进来的字段发给后端 API，
/// 跟 expense-tracker-api 那边"局部更新"的逻辑保持一致：
/// 没传的字段（None）会因为 skip_serializing_if 被整体跳过，
/// 不会被序列化成 "字段": null 发过去，也就不会误把没提到的字段清空
#[derive(Debug, Serialize)]
struct UpdateExpenseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
}

// ---------- MCP Server 本体 ----------

// 这个 Server 不连数据库，它只是 expense-tracker-api 这个
// axum web 服务的一个"翻译层"：每个 MCP 工具方法内部
// 其实就是发一次 HTTP 请求过去，把结果包装成 MCP 要求的格式返回
#[derive(Clone)]
struct ExpenseServer {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ExpenseServer {
    fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            // 优先读环境变量，读不到就用本地默认值，
            // 这样以后部署到别的地方也不用改代码
            base_url: std::env::var("EXPENSE_API_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            api_key: std::env::var("EXPENSE_API_KEY")
                .unwrap_or_else(|_| "dev-secret-key".to_string()),
        }
    }

    /// 六个工具方法共用的"响应处理"逻辑，避免每个方法都写一遍
    /// 同样的判断。核心思路：
    /// - 请求本身失败（网络错误等）→ 包装成 McpError
    /// - 请求发出去了，但 API 返回非 2xx（比如 404 / 401 / 400）→ 也算 McpError
    /// - 只有 2xx 才算成功，把响应体原样包成 CallToolResult 返回
    async fn respond(
        result: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<CallToolResult, McpError> {
        match result {
            Ok(resp) => {
                let status = resp.status();
                let body = resp
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("读取响应内容失败: {e}"));

                if status.is_success() {
                    Ok(CallToolResult::success(vec![ContentBlock::text(body)]))
                } else {
                    Err(McpError::internal_error(
                        format!("expense-tracker-api 返回了 {status}: {body}"),
                        None,
                    ))
                }
            }
            Err(e) => Err(McpError::internal_error(
                format!("请求 expense-tracker-api 失败: {e}"),
                None,
            )),
        }
    }
}

// #[tool_router(server_handler)] 是"单 impl 块"写法：
// 不需要额外再写一个 impl ServerHandler for ExpenseServer，
// 宏会把这里标了 #[tool] 的方法自动收集成一份工具清单
#[tool_router(server_handler)]
impl ExpenseServer {
    #[tool(description = "List expenses, optionally filtered by category and/or month (YYYY-MM)")]
    async fn list_expenses(
        &self,
        Parameters(p): Parameters<ListExpensesParams>,
    ) -> Result<CallToolResult, McpError> {
        // 只有用户真的传了 category / month，才拼到查询参数里
        let mut query = vec![];
        if let Some(category) = &p.category {
            query.push(("category".to_string(), category.clone()));
        }
        if let Some(month) = &p.month {
            query.push(("month".to_string(), month.clone()));
        }

        let result = self
            .http
            .get(format!("{}/expenses", self.base_url))
            .header("x-api-key", &self.api_key)
            .query(&query)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(description = "Get a single expense by its id")]
    async fn get_expense(
        &self,
        Parameters(p): Parameters<GetExpenseParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .http
            .get(format!("{}/expenses/{}", self.base_url, p.id))
            .header("x-api-key", &self.api_key)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(description = "Create a new expense")]
    async fn create_expense(
        &self,
        Parameters(p): Parameters<CreateExpenseParams>,
    ) -> Result<CallToolResult, McpError> {
        // p 本身就是要发的 JSON body，字段名和 API 那边的
        // CreateExpense 结构体是对得上的，直接 .json(&p) 就行
        let result = self
            .http
            .post(format!("{}/expenses", self.base_url))
            .header("x-api-key", &self.api_key)
            .json(&p)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(
        description = "Partially update an existing expense — only send the fields that should change"
    )]
    async fn update_expense(
        &self,
        Parameters(p): Parameters<UpdateExpenseParams>,
    ) -> Result<CallToolResult, McpError> {
        // 这里特意从 UpdateExpenseParams 转成单独的 UpdateExpenseBody，
        // 是因为 body 需要 skip_serializing_if 来跳过没填的字段，
        // 而 id 只是用来拼 URL 路径的，不应该出现在请求体里
        let body = UpdateExpenseBody {
            description: p.description,
            amount: p.amount,
            category: p.category,
            date: p.date,
        };

        let result = self
            .http
            .put(format!("{}/expenses/{}", self.base_url, p.id))
            .header("x-api-key", &self.api_key)
            .json(&body)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(description = "Delete an expense by its id")]
    async fn delete_expense(
        &self,
        Parameters(p): Parameters<DeleteExpenseParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .http
            .delete(format!("{}/expenses/{}", self.base_url, p.id))
            .header("x-api-key", &self.api_key)
            .send()
            .await;

        Self::respond(result).await
    }

    #[tool(
        description = "Get total spending and a per-category breakdown, optionally for one month (YYYY-MM)"
    )]
    async fn get_summary(
        &self,
        Parameters(p): Parameters<GetSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut query = vec![];
        if let Some(month) = &p.month {
            query.push(("month".to_string(), month.clone()));
        }

        let result = self
            .http
            .get(format!("{}/expenses/summary", self.base_url))
            .header("x-api-key", &self.api_key)
            .query(&query)
            .send()
            .await;

        Self::respond(result).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 日志要写到 stderr，不能写到 stdout —— 因为 stdio 传输方式下，
    // stdout 是留给 MCP 协议本身通信用的，混进普通日志会把协议搞坏
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let server = ExpenseServer::new();
    // 用 stdio 传输方式启动：这个进程会被 Client 当作子进程拉起，
    // 通过标准输入输出跟 Client 交换消息，跟我们幻灯片里讲的一致
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}