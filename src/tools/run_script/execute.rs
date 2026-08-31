use tokio::process::Command;

/// 通用执行逻辑：把命令交给 `sh -c` 执行，支持任意 python/node/bash 命令。
/// 返回 exit_code + stdout + stderr 的汇总文本。
///
/// 安全说明：本函数能执行任意命令(RCE)，防护完全依赖上层 ApprovalCallback 的
/// 人工确认——每次执行前都会把完整命令展示给用户、并高亮潜在危险片段，
/// 由用户 y/n 决定是否放行。（这是演示用途，非严肃生产环境。）
pub async fn run_script(command: &str, working_dir: Option<&str>) -> anyhow::Result<String> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("命令启动失败：{e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(-1);

    Ok(format!(
        "exit_code: {code}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        stdout.trim_end(),
        stderr.trim_end()
    ))
}
