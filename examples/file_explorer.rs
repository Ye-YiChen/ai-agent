// 示例：文件探索 Agent（解压包 -> 找出最合适候选人 -> 删掉用完的文件）
// 步骤：解压 zip -> list_files 看目录 -> read_file/read_image 逐一确认 -> 得出结论 -> 删除指定文件
// 用到的功能：
//   - Agent::run：多步自主探索（对话走 DeepSeek，上限 20 步）
//   - build_file_explorer_toolbox：unzip / list / read / read_image / delete 工具
//   - ApprovalCallback：delete_file 等高危操作先过人工审批
//   - SearchCompressorCallback：超长结果自动压缩
//   - VISION_MODEL：读图走 OpenRouter 多模态模型
use std::sync::Arc;

use ai_agent::{agent::Agent, callback::{approval::ApprovalCallback, search_compressor::SearchCompressorCallback}, constant::{DEEPSEEK_FLASH, VISION_MODEL}, tools::build_file_explorer_toolbox};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let toolbox = Arc::new(build_file_explorer_toolbox(VISION_MODEL));

    let instructions = r#"你是一位善于探索文件的助手。
拿到一个压缩包时，先解压，再用 list_files 看目录结构，
靠文件名判断哪些文件可能相关，用 read_file 或 read_image 逐一确认，
最后再给出结论。不要跳过探索步骤直接猜答案。"#;

    let agent = Agent::new(DEEPSEEK_FLASH, Some(instructions), toolbox)
        .with_max_steps(20)
        // 高危操作先过人工审批 —— delete_file 在名单里，其他文件工具都不需要审批
        .with_before_tool_callback(Arc::new(ApprovalCallback::new(["delete_file"])))
        // web_search 结果太长时自动向量压缩，不用每次手动处理
        .with_after_tool_callback(Arc::new(SearchCompressorCallback));

    let result = agent
        .run(r#"读一下这个压缩包里的候选人信息和职位要求，判断出最合适的候选人。
确认完之后，把 job_description.txt 删掉，因为信息已经用完了，不需要再占地方。
压缩包路径：/Users/dave/Desktop/example/candidates.zip"#)
        .await?;

    println!("回答: {}", result.output);
    println!(
        "\n本次执行一共走了 {} 步，记录了 {} 条 Event",
        result.context.current_step,
        result.context.events.len()
    );

    Ok(())
}
