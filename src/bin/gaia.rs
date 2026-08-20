use std::sync::Arc;

use ai_agent::{
    constant::{DEEPSEEK_FLASH, VISION_MODEL},
    gaia::{
        dataset::load_gaia_level1,
        evaluator::evaluate_gaia_single_with_tools,
        models::GaiaEvalResult,
    },
    llm::semaphore::get_semaphore,
    skill::load_skills,
    tools::build_full_toolbox,
};
use tokio::task::JoinSet;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    gaia_level1_experiment().await
}

pub async fn gaia_level1_experiment() -> anyhow::Result<()> {
    let problems = load_gaia_level1().await?;
    let skills = Arc::new(load_skills("skills"));
    let toolbox = Arc::new(build_full_toolbox(VISION_MODEL, skills).await?);

    let mut set = JoinSet::new();

    for problem in problems.iter() {
        let problem = problem.clone();
        let toolbox = toolbox.clone();
        set.spawn(async move {
            let permit = get_semaphore().acquire().await?;
            let eval = evaluate_gaia_single_with_tools(problem, DEEPSEEK_FLASH, toolbox).await;
            drop(permit);
            Ok::<_, anyhow::Error>(eval)
        });
    }

    let mut results: Vec<GaiaEvalResult> = Vec::new();
    while let Some(Ok(result)) = set.join_next().await {
        match result {
            Ok(eval) => {
                tracing::info!("{eval:#?}");
                results.push(eval);
            }
            Err(e) => tracing::error!("task panicked: {e}"),
        }
    }

    let correct = results.iter().filter(|e| e.correct).count();
    let total = results.len();
    tracing::info!(
        "=== 带工具评测 ===\nwith_tools: {correct}/{total} ({:.1}%)",
        correct as f64 / total as f64 * 100.0
    );

    // 失败原因汇总：区分"执行报错"与"答错(答案不匹配)"两类
    let failures: Vec<&GaiaEvalResult> = results.iter().filter(|e| !e.correct).collect();
    if !failures.is_empty() {
        let mut error_count = 0usize;
        let mut wrong_count = 0usize;
        let mut lines = String::new();
        for f in &failures {
            let short_id: String = f.task_id.chars().take(8).collect();
            match &f.error {
                Some(err) => {
                    error_count += 1;
                    lines.push_str(&format!("\n  [报错] {short_id}: {err}"));
                }
                None => {
                    wrong_count += 1;
                    lines.push_str(&format!(
                        "\n  [答错] {short_id}: 预测={:?} 正确={:?}",
                        f.prediction.as_deref().unwrap_or("(空)"),
                        f.answer
                    ));
                }
            }
        }
        tracing::info!(
            "=== 失败原因汇总（共 {} 题失败：执行报错 {} / 答案错误 {}）==={}",
            failures.len(),
            error_count,
            wrong_count,
            lines
        );
    }

    Ok(())
}
