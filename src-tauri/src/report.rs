use crate::llm::{Provider, Provider as P};
use crate::store::{Report, Store};
use chrono::{Datelike, Local, Timelike};

pub fn period_bounds(period: &str) -> (i64, i64) {
    let now = Local::now();
    let today = now.date_naive();
    let start_date = match period {
        "week" => {
            let weekday = now.weekday().num_days_from_monday() as i64;
            today - chrono::Duration::days(weekday)
        }
        "month" => today.with_day(1).unwrap_or(today),
        _ => today,
    };
    let start = start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp_millis();
    (start, now.timestamp_millis())
}

fn fmt_time(ts: i64) -> String {
    use chrono::TimeZone;
    let dt = chrono::Local.timestamp_millis_opt(ts).single();
    match dt {
        Some(t) => format!("{:02}:{:02}", t.hour(), t.minute()),
        None => "--:--".to_string(),
    }
}

fn build_segments_text(segments: &[crate::store::WorkSegment]) -> String {
    let mut out = String::new();
    for s in segments.iter() {
        let mins = ((s.ts_end - s.ts_start) / 60000).max(1);
        let app = s.category.clone().unwrap_or_else(|| "未知".to_string());
        let summary = s.summary.clone().unwrap_or_default();
        out.push_str(&format!(
            "[{}-{}] {} | {} ({}分钟)\n",
            fmt_time(s.ts_start),
            fmt_time(s.ts_end),
            app,
            summary,
            mins
        ));
    }
    out
}

pub async fn generate_report(
    store: &Store,
    provider: &P,
    period: &str,
    extra: &str,
) -> Result<Report, String> {
    let (start, end) = period_bounds(period);
    let gap: i64 = store
        .get_setting("idle_threshold_ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(90000);
    crate::aggregator::aggregate_and_store(store, start, end, gap)
        .map_err(|e| format!("聚合失败: {}", e))?;

    let segments = store.list_segments(start, end);
    let segments_text = build_segments_text(&segments);

    let prompt = store.get_template_prompt(1).unwrap_or_else(|| crate::store::DEFAULT_DAILY_PROMPT.to_string());
    let user = prompt
        .replace("{segments}", &segments_text)
        .replace("{extra}", extra);

    let system = "你是 Daily Trace，一个理解用户工作并生成专业日报的 AI。";
    let (content, model) = provider
        .chat(system, &user)
        .await
        .map_err(|e| format!("LLM 调用失败: {}", e))?;

    let seg_ids: Vec<i64> = segments.iter().map(|s| s.id).collect();
    let seg_ids_json = serde_json::to_string(&seg_ids).unwrap_or_default();
    let id = store
        .insert_report(period, start, end, Some(1), &content, &seg_ids_json, &model)
        .map_err(|e| format!("保存报告失败: {}", e))?;

    Ok(Report {
        id,
        period: period.to_string(),
        period_start: start,
        period_end: end,
        content,
        model: Some(model),
        created_at: chrono::Utc::now().timestamp_millis(),
    })
}

pub fn build_provider_from_settings(store: &Store) -> Provider {
    let kind = store.get_setting("model_kind").unwrap_or_else(|| "mock".to_string());
    match kind.as_str() {
        "openai" | "custom" => {
            let base_url = store
                .get_setting("model_base_url")
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
            let api_key = store.get_setting("model_api_key").unwrap_or_default();
            let model = store
                .get_setting("model_name")
                .unwrap_or_else(|| "gpt-4o-mini".to_string());
            Provider::OpenAI {
                base_url,
                api_key,
                model,
            }
        }
        _ => Provider::Mock,
    }
}
