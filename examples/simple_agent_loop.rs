use ai_agent::{constant::GPT_4O_MINI_MODEL, llm::complete::chat_complete, tools::build_toolbox};
use chrono::Local;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // build_toolbox 现在是 async 的：它会顺便把 expense_mcp_server
    // 拉起来、握手、拿到它暴露的工具列表，跟 calculator / web_search
    // 一起塞进同一个工具箱里
    let toolbox = build_toolbox().await?;

    // 当前时间
    let now = Local::now();
    let current_time = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let system_prompt = format!(
        r#"你是一位专业、可靠、乐于帮助用户的 AI 助手。

当前本地时间：{}

请始终将"今天"、"昨天"、"明天"、"本周"、"本月"、"上个月"等相对时间，
解释为相对于上面的当前时间。

你可以使用多个工具来帮助完成任务。

工具使用原则：

1. 如果问题可以直接回答，则直接回答，不要调用工具。

2. 如果用户的问题需要最新的信息，例如：
   - 新闻
   - 天气
   - 汇率
   - 股票
   - 网络搜索
   等，请使用 Web Search 工具。

3. 如果需要进行数学计算、金额计算、百分比计算、
   或者任何要求结果精确的计算，请使用 Calculator 工具。

4. 当用户需要查询、统计、新增、修改、删除费用记录时，
   请使用 Expense MCP 提供的工具，例如：
   - create_expense
   - list_expenses
   - get_summary
   等。

5. 不要猜测工具可以提供的数据。

6. 如果工具能够得到答案，就应该调用工具，
   不要回答"我不知道"。

7. 工具返回结果以后，请直接根据工具结果生成自然、简洁、准确的回答，
   不要把工具调用过程告诉用户。

请始终优先完成用户的任务，而不是刻意调用工具。"#,
        current_time
    );

    println!("\n=== 测试 0：Agent loop 测试 ===");
    let result = chat_complete(
        GPT_4O_MINI_MODEL,
        Some(&system_prompt),
        r"我想买一台 Mac Mini M4。

请帮我做一个购买分析：

1. 使用搜索工具查询目前 Mac Mini M4 的价格。
2. 查询我过去三个月的 Software 分类支出。
3. 计算：
   - Mac Mini M4 价格占我过去三个月 Software 支出的多少倍。
   - 如果我每个月节省500元，需要多少个月才能攒够购买它。
4. 根据我的消费情况，给我一个是否应该购买的建议。

所有价格和消费数据必须来自工具。
不要自己猜测数据。",
        &toolbox,
    )
    .await?;
    println!("回答: {result}");

    // // 测试 1：查询某个月某个分类的花销（应该会触发 get_summary 或 list_expenses）
    // println!("\n=== 测试 1：查询七月 Food 分类花销 ===");
    // let result = chat_complete(
    //     GPT_4O_MINI_MODEL,
    //     Some(&system_prompt),
    //     "我七月在 Food 这个分类上一共花了多少钱？",
    //     &toolbox,
    // )
    // .await?;
    // println!("回答: {result}");

    // // 测试 2：新增一笔支出（应该会触发 create_expense）
    // println!("\n=== 测试 2：新增一笔支出 ===");
    // let result = chat_complete(
    //     GPT_4O_MINI_MODEL,
    //     Some(&system_prompt),
    //     "帮我记一笔支出：今天在星巴克买咖啡花了 28 块钱，分类是 Food。然后在帮我统计一下七月份在food这个分类上的开销。",
    //     &toolbox,
    // )
    // .await?;
    // println!("回答: {result}");

    // // 测试 3：查看七月整体费用汇总（应该会触发 get_summary，
    // // 而且能看到测试 2 新增的那一笔也算进去了 —— 因为
    // // expense-tracker-api 的数据在进程内存里是共享的，
    // // 不会因为每次请求而重置）
    // println!("\n=== 测试 3：七月整体费用汇总 ===");
    // let result = chat_complete(
    //     GPT_4O_MINI_MODEL,
    //     Some(&system_prompt),
    //     "帮我总结一下七月的费用情况，按分类列出来。",
    //     &toolbox,
    // )
    // .await?;
    // println!("回答: {result}");

    Ok(())
}
