use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{agent::ExecutionContext, tools::{file_read::execute::read, tool::Tool}};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadFileArgs {
    pub file_path: String,

    #[serde(default = "default_start")]
    pub start_line: usize,

    #[serde(default = "default_end")]
    pub end_line: i64
}

fn default_start() -> usize {
    1
}

fn default_end() -> i64 {
    -1
}

pub struct ReadFileTool;

#[async_trait::async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read a text file (returned with line numbers, optionally a range) or a CSV file (returned as a markdown table)."
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(ReadFileArgs)).expect("schema is always serializable")
    }

    async fn execute(&self, args_json: &str, _context: &ExecutionContext) -> anyhow::Result<String> {
        let args: ReadFileArgs = serde_json::from_str(args_json)?;
        match read(&args) {
            Ok(content) => Ok(content),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}