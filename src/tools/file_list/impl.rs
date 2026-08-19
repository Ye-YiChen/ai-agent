use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{agent::ExecutionContext, tools::{file_list::execute::list, tool::Tool}};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListFilesArgs {
    #[serde(default = "default_path")]
    pub path: String,
}

fn default_path() -> String {
    ".".to_string()
}

pub struct ListFileTool;

#[async_trait::async_trait]
impl Tool for ListFileTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "List files and directories at a given path, directories listed first."
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(ListFilesArgs)).expect("schema is always serializable")
    }

    async fn execute(&self, args_json: &str, _context: &ExecutionContext) -> anyhow::Result<String> {
        let args: ListFilesArgs = serde_json::from_str(args_json)?;
        match list(&args.path) {
            Ok(listing) => Ok(listing),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}