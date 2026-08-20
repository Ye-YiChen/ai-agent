use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use serde_json::json;

pub fn web_search_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObjectArgs::default()
            .name("web_search")
            .description("Perform web search operations.")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query to execute with Tavily."
                    },
                    "max_results": {
                        "type": "number",
                        "description": "The maximum number of search results to return.",
                        "default": 5,
                        "minimum": 0,
                        "maximum": 20
                    },
                    "topic": {
                        "type": "string",
                        "description": "Use this to specify the topic of the search. Options are 'general', 'news', or 'finance'.",
                        "enum": ["general", "news", "finance"],
                        "default": "general"
                    },
                    "time_range": {
                        "type": "string",
                        "description": "Restrict the search results to a specific time range. Options are 'day', 'week', 'month', or 'year'.",
                        "enum": ["day", "week", "month", "year"],
                    }
                },
                "required": ["query"]
            }))
            .build()
            .expect("Failed to build web_search function object"),
    })
}
