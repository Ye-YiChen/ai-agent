use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{agent::ExecutionContext, tools::{file_unzip::execute::unzip, tool::Tool}};

#[derive(Deserialize, Debug, JsonSchema)]
pub struct UnzipFileArgs {
    pub zip_path: String,

    #[serde(default)]
    pub extract_to: Option<String>,
}

pub struct UnzipFileTool;

#[async_trait::async_trait]
impl Tool for UnzipFileTool {
    fn name(&self) -> &str {
        "unzip_file"
    }

    fn description(&self) -> &str {
        "Extract a zip archive so its contents can be explored with list_files and read_file."
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(UnzipFileArgs))
            .expect("schema is always serializable")
    }

    async fn execute(
        &self,
        args_json: &str,
        _context: &ExecutionContext,
    ) -> anyhow::Result<String> {
        let args: UnzipFileArgs = serde_json::from_str(args_json)?;
        match unzip(&args.zip_path, args.extract_to.as_deref()) {
            Ok(summary) => Ok(summary),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}
