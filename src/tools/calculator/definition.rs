use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use serde_json::json;

pub fn calculator_tool_definition() -> ChatCompletionTools{
    ChatCompletionTools::Function(ChatCompletionTool{
        function: FunctionObjectArgs::default()
            .name("calculator")
            .description("Perform basic arithmetic operations.")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "operator": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"],
                        "description": "The arithmetic operation to perform."
                    },
                    "first_number": {
                        "type": "number",
                        "description": "The first number in the operation."
                    },
                    "second_number": {
                        "type": "number",
                        "description": "The second number in the operation."
                    }
                },
                "required": ["operator", "first_number", "second_number"]
            }))
            .build()
            .expect("Failed to build calculator function object"),
    })
}