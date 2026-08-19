// 之前 Agent -> 本地Tool -> calculator/web_search
// 现在 Agent -> ToolBox -> 本地工具
//                      -> MCP Client -> MCP Server -> expense-tracker-api(模拟)


pub mod client;
pub mod tool;