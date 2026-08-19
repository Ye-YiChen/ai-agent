use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{agent::ExecutionContext, tools::{read_image::execute::analyze_image, tool::Tool}};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadImageArgs {
    pub file_path: String,
    pub query: String,
}

pub struct ReadImageTool {
    model: String,
}

impl ReadImageTool {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for ReadImageTool {
fn name(&self) -> &str {
        "read_image"
    }

    fn description(&self) -> &str {
        "Analyze an image file with a vision model to answer a question about what it shows."
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(ReadImageArgs)).expect("schema is always serializable")
    }

    async fn execute(&self, args_json: &str, _context: &ExecutionContext) -> anyhow::Result<String> {
        let args: ReadImageArgs = serde_json::from_str(args_json)?;
        match analyze_image(&args.file_path, &args.query, &self.model).await {
            Ok(answer) => Ok(answer),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}
