use serde_json::Value;

use crate::{
    agent::{ContentItem, ExecutionContext, ToolResultStatus, callback::AfterToolCallback},
    knowledge_base::{chunk::fixed_length_chunking, search::vector_search},
};

const COMPRESS_THRESHOLD: usize = 2000; // 结果超过该字符数才考虑压缩
const CHUNK_SIZE: usize = 500;
const CHUNK_OVERLAP: usize = 50;
const TOP_K: usize = 3;
const HARD_LIMIT: usize = 12000; // 向量压缩不可用时的硬截断上限（字符），绝对防止上下文爆炸

/// 工具结果压缩回调：对**任意工具**的超长成功结果进行压缩，防止长内容
/// （web_search / run_script / read_file / MCP 等）原样滚入对话历史导致上下文爆炸。
///
/// 策略：
/// 1. 优先向量检索压缩——把结果切块，按"查询"取最相关的 Top-K 块（需要 embedding 可用）。
///    查询来源：该工具调用自带的 query 参数（如 web_search），否则退回最近一次用户提问。
/// 2. 兜底硬截断——向量压缩拿不到查询或失败时，保留头尾、中间省略，确保不超上限。
pub struct SearchCompressorCallback;

#[async_trait::async_trait]
impl AfterToolCallback for SearchCompressorCallback {
    async fn call(
        &self,
        context: &ExecutionContext,
        tool_call_id: &str,
        tool_name: &str,
        status: ToolResultStatus,
        content: &str,
    ) -> Option<(ToolResultStatus, String)> {
        // 只处理成功且超长的结果；覆盖所有工具（不再限于 web_search）
        if status != ToolResultStatus::Success {
            return None;
        }
        let char_count = content.chars().count();
        if char_count < COMPRESS_THRESHOLD {
            return None;
        }

        // 取相关性检索用的 query：优先该工具调用自带的 query 参数，否则退回最近一次用户提问
        let query = extract_query(context, tool_call_id).or_else(|| latest_user_message(context));

        if let Some(query) = query {
            let chunks = fixed_length_chunking(content, CHUNK_SIZE, CHUNK_OVERLAP);
            if !chunks.is_empty() {
                tracing::info!(
                    "🔍 Compressing {tool_name} result: {char_count} chars → {} chunks...",
                    chunks.len(),
                );
                match vector_search(&query, &chunks, TOP_K).await {
                    Ok(hits) => {
                        let compressed = hits
                            .into_iter()
                            .map(|hit| hit.text)
                            .collect::<Vec<_>>()
                            .join("\n\n");
                        tracing::info!(
                            "✅ Compressed {tool_name}: {char_count} → {} chars",
                            compressed.chars().count(),
                        );
                        return Some((status, compressed));
                    }
                    Err(err) => {
                        tracing::warn!(
                            "Vector compression failed ({err}), fallback to truncation"
                        );
                    }
                }
            }
        }

        // 兜底：向量压缩不可用/失败时硬截断，绝对防止上下文爆炸
        if char_count > HARD_LIMIT {
            let truncated = truncate_middle(content, HARD_LIMIT);
            tracing::info!(
                "✂️  Truncated {tool_name} result: {char_count} → {} chars",
                truncated.chars().count(),
            );
            return Some((status, truncated));
        }

        None
    }
}

/// 取该 tool_call 的 query 参数（不限工具名，只要 arguments 里带 query 字段）。
fn extract_query(context: &ExecutionContext, tool_call_id: &str) -> Option<String> {
    context
        .events
        .iter()
        .flat_map(|event| &event.content)
        .find_map(|item| match item {
            ContentItem::ToolCall {
                tool_call_id: id,
                arguments,
                ..
            } if id == tool_call_id => arguments
                .get("query")
                .and_then(Value::as_str)
                .map(str::to_owned),
            _ => None,
        })
}

/// 退而求其次：用最近一次用户消息作为相关性检索的 query。
fn latest_user_message(context: &ExecutionContext) -> Option<String> {
    context
        .events
        .iter()
        .rev()
        .flat_map(|event| event.content.iter().rev())
        .find_map(|item| match item {
            ContentItem::Message { role, content } if role == "user" => Some(content.clone()),
            _ => None,
        })
}

/// 保留头部与尾部、中间省略的截断（按字符安全切分，避免截断多字节字符）。
fn truncate_middle(text: &str, limit: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= limit {
        return text.to_string();
    }
    let head = limit * 2 / 3;
    let tail = limit - head;
    let omitted = chars.len() - head - tail;
    let head_str: String = chars[..head].iter().collect();
    let tail_str: String = chars[chars.len() - tail..].iter().collect();
    format!("{head_str}\n\n...[已省略 {omitted} 个字符]...\n\n{tail_str}")
}
