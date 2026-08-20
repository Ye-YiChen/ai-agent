use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{agent::ExecutionContext, tools::tool::Tool};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteFileArgs {
    /// 要写入的目标文件路径（父目录不存在会自动创建）
    pub file_path: String,

    /// 要写入的完整文本内容
    pub content: String,

    /// 是否追加写入（true=在文件末尾追加，false=覆盖整个文件）。默认 false。
    #[serde(default)]
    pub append: bool,
}

pub struct WriteFileTool;

#[async_trait::async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "写入文本到文件。默认覆盖整个文件；append=true 时在末尾追加。父目录不存在会自动创建。"
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(WriteFileArgs))
            .expect("schema is always serializable")
    }

    async fn execute(
        &self,
        args_json: &str,
        _context: &ExecutionContext,
    ) -> anyhow::Result<String> {
        let args: WriteFileArgs = serde_json::from_str(args_json)?;

        match super::execute::write(&args).await {
            Ok(bytes) => {
                let mode = if args.append { "追加" } else { "写入" };
                tracing::info!("✅ 已{mode} {} 字节到 {}", bytes, args.file_path);
                Ok(format!("已{mode} {bytes} 字节到 {}", args.file_path))
            }
            Err(e) => Err(anyhow::anyhow!("写入 {} 失败: {e}", args.file_path)),
        }
    }
}
