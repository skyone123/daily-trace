use crate::llm::Provider;
use crate::store::{Store, Todo, WorkSegment};

fn title_from_summary(s: &str) -> Option<String> {
    s.split_once('「')
        .and_then(|(_, a)| a.split_once('」').map(|(t, _)| t.to_string()))
}

fn segments_text(segments: &[WorkSegment]) -> String {
    segments
        .iter()
        .map(|s| {
            let app = s.category.clone().unwrap_or_else(|| "未知".to_string());
            let sum = s.summary.clone().unwrap_or_default();
            format!("- {} | {}", app, sum)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn mock_extract_todos(segments: &[WorkSegment]) -> Vec<(String, Option<i64>)> {
    use std::collections::HashSet;
    let mut out = Vec::new();
    let mut seen_apps: HashSet<String> = HashSet::new();
    for s in segments {
        let app = s.category.clone().unwrap_or_default();
        if app.is_empty() || !seen_apps.insert(app.clone()) {
            continue;
        }
        let title = title_from_summary(&s.summary.clone().unwrap_or_default());
        let todo_title = match app.as_str() {
            "VS Code" => format!(
                "完成 {} 的待实现逻辑",
                title.as_deref().unwrap_or("代码")
            ),
            "Chrome" => "整理今日调研资料并归档要点".to_string(),
            "微信" => "跟进工作群中的待回复事项".to_string(),
            "Notion" => format!("完善 {}", title.as_deref().unwrap_or("文档")),
            "Terminal" => "确认构建与测试任务已通过".to_string(),
            "Figma" => "评审首页设计稿并给出反馈".to_string(),
            _ => format!("推进 {} 相关工作", app),
        };
        out.push((todo_title, Some(s.id)));
        if out.len() >= 8 {
            break;
        }
    }
    out
}

fn mock_evaluate(title: &str, recent: &[WorkSegment]) -> bool {
    let t = title.to_lowercase();
    for s in recent {
        if let Some(tt) = title_from_summary(&s.summary.clone().unwrap_or_default()) {
            let tl = tt.to_lowercase();
            if !tl.is_empty() && t.contains(&tl) {
                return true;
            }
        }
    }
    false
}

fn parse_todos_lines(out: &str, segments: &[WorkSegment]) -> Vec<(String, Option<i64>)> {
    let first_seg = segments.first().map(|s| s.id);
    out.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| {
            l.trim_start_matches(|c: char| {
                c.is_ascii_digit() || c == '.' || c == ' ' || c == '-' || c == '*' || c == '【' || c == '['
            })
            .trim()
            .trim_end_matches('】')
            .trim_end_matches(']')
            .to_string()
        })
        .filter(|l| !l.is_empty() && l.chars().count() > 2)
        .take(8)
        .map(|t| (t, first_seg))
        .collect()
}

pub async fn generate_todos(
    store: &Store,
    provider: &Provider,
    period: &str,
) -> Result<Vec<Todo>, String> {
    let (start, end) = crate::report::period_bounds(period);
    let gap = store
        .get_setting("idle_threshold_ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(90000);
    crate::aggregator::aggregate_and_store(store, start, end, gap)
        .map_err(|e| e.to_string())?;
    let segments = store.list_segments(start, end);
    if segments.is_empty() {
        return Err("该周期无工作记录，无法提取待办".to_string());
    }
    let candidates: Vec<(String, Option<i64>)> = match provider {
        Provider::Mock => mock_extract_todos(&segments),
        Provider::OpenAI { .. } => {
            let text = segments_text(&segments);
            let system = "你是待办提取助手。从工作记录中提取候选待办事项，每行一个标题，中文，简洁可执行，不超过 8 条。不要编号、不要解释、不要前后缀。";
            let user = format!("工作记录：\n{}\n\n请提取待办。", text);
            let (out, _) = provider
                .chat(system, &user)
                .await
                .map_err(|e| e.to_string())?;
            parse_todos_lines(&out, &segments)
        }
    };
    for (title, seg_id) in &candidates {
        let _ = store.insert_todo(title, None, None, None, *seg_id);
    }
    Ok(store.list_todos(None, 200))
}

pub async fn evaluate_todos(store: &Store, provider: &Provider) -> Result<usize, String> {
    let open = store.list_todos(Some("open"), 100);
    if open.is_empty() {
        return Ok(0);
    }
    let now = chrono::Utc::now().timestamp_millis();
    let from = now - 24 * 3600 * 1000;
    let recent = store.list_segments(from, now);
    let recent_text = segments_text(&recent);
    let mut updated = 0usize;
    for t in &open {
        let done: bool = match provider {
            Provider::Mock => mock_evaluate(&t.title, &recent),
            Provider::OpenAI { .. } => {
                let system = "你是待办完成度评估助手。根据最近工作记录判断该待办是否已被推进或完成。只回答一个词：done 或 open。";
                let user = format!("待办：{}\n\n最近工作记录：\n{}", t.title, recent_text);
                let (out, _) = provider
                    .chat(system, &user)
                    .await
                    .map_err(|e| e.to_string())?;
                out.trim().to_lowercase().starts_with("done")
            }
        };
        if done {
            store
                .update_todo_status(t.id, "done")
                .map_err(|e| e.to_string())?;
            updated += 1;
        }
    }
    Ok(updated)
}
