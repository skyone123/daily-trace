use crate::capture::MockSource;
use crate::capture::CaptureSource;
use crate::report;
use crate::state::AppState;
use crate::store::{Report, TimelineEvent, Todo, WorkSegment};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tauri::State;

#[derive(Serialize)]
pub struct AppStat {
    pub app: String,
    pub ms: i64,
}

#[tauri::command]
pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[tauri::command]
pub fn list_timeline(
    from: i64,
    to: i64,
    limit: Option<i64>,
    state: State<AppState>,
) -> Vec<TimelineEvent> {
    state.store.list_events(from, to, limit.unwrap_or(2000))
}

#[tauri::command]
pub fn list_segments(from: i64, to: i64, state: State<AppState>) -> Vec<WorkSegment> {
    state.store.list_segments(from, to)
}

#[tauri::command]
pub fn aggregate_range(
    from: i64,
    to: i64,
    state: State<AppState>,
) -> Result<usize, String> {
    let gap = state
        .store
        .get_setting("idle_threshold_ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(90000);
    crate::aggregator::aggregate_and_store(&state.store, from, to, gap)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_report(
    period: String,
    extra: String,
    state: State<'_, AppState>,
) -> Result<Report, String> {
    let provider = report::build_provider_from_settings(&state.store);
    report::generate_report(&state.store, &provider, &period, &extra).await
}

#[tauri::command]
pub fn list_reports(limit: Option<i64>, state: State<AppState>) -> Vec<Report> {
    state.store.list_reports(limit.unwrap_or(50))
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> HashMap<String, String> {
    state.store.list_all_settings()
}

#[tauri::command]
pub fn save_setting(key: String, value: String, state: State<AppState>) -> Result<(), String> {
    state
        .store
        .set_setting(&key, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_paused(paused: bool, state: State<AppState>) -> Result<(), String> {
    state.collector.paused.store(paused, Ordering::Relaxed);
    let _ = state
        .store
        .set_setting("paused", if paused { "true" } else { "false" });
    if paused {
        let now = chrono::Utc::now().timestamp_millis();
        let _ = state.store.close_last_open_event(now);
    }
    Ok(())
}

#[tauri::command]
pub fn list_categories(state: State<AppState>) -> Vec<String> {
    state.store.list_categories()
}

#[tauri::command]
pub fn stats_by_app(from: i64, to: i64, state: State<AppState>) -> Vec<AppStat> {
    state
        .store
        .stats_by_app(from, to)
        .into_iter()
        .map(|(app, ms)| AppStat { app, ms })
        .collect()
}

#[tauri::command]
pub fn seed_demo_data(state: State<AppState>) -> Result<usize, String> {
    let (day_start, now) = report::period_bounds("day");
    let nine = day_start + 9 * 3600 * 1000;
    let start_ts = nine.max(day_start);
    let end_ts = now.max(start_ts + 3600_000);
    let mock = MockSource::new();
    let mut ts = start_ts;
    let mut count = 0usize;
    while ts < end_ts {
        let act = mock.current_activity().unwrap();
        let dur_min = 20 + ((ts / 60000) % 30);
        let dur_ms = dur_min * 60 * 1000;
        let seg_end = (ts + dur_ms).min(end_ts);
        let _ = state.store.insert_event(
            ts,
            Some(seg_end),
            "app",
            Some(&act.app_name),
            Some(&act.window_title),
            None,
            None,
        );
        ts = seg_end + 5 * 60 * 1000;
        count += 1;
        if count > 200 {
            break;
        }
    }
    Ok(count)
}

#[tauri::command]
pub fn list_todos(status: Option<String>, state: State<AppState>) -> Vec<Todo> {
    state.store.list_todos(status.as_deref(), 500)
}

#[tauri::command]
pub async fn generate_todos(
    period: String,
    state: State<'_, AppState>,
) -> Result<Vec<Todo>, String> {
    let provider = report::build_provider_from_settings(&state.store);
    crate::todo::generate_todos(&state.store, &provider, &period).await
}

#[tauri::command]
pub fn update_todo(id: i64, status: String, state: State<AppState>) -> Result<(), String> {
    state
        .store
        .update_todo_status(id, &status)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn evaluate_todos(state: State<'_, AppState>) -> Result<usize, String> {
    let provider = report::build_provider_from_settings(&state.store);
    crate::todo::evaluate_todos(&state.store, &provider).await
}

#[tauri::command]
pub async fn classify_segments(
    from: i64,
    to: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let provider = report::build_provider_from_settings(&state.store);
    crate::classify::classify_segments(&state.store, &provider, from, to).await
}

#[tauri::command]
pub fn export_data(state: State<AppState>) -> String {
    serde_json::to_string_pretty(&state.store.export_data()).unwrap_or_default()
}

#[tauri::command]
pub fn stats_heatmap(
    from: i64,
    to: i64,
    state: State<AppState>,
) -> Vec<crate::stats::HeatCell> {
    let segs = state.store.list_segments(from, to);
    crate::stats::heatmap(&segs)
}

#[tauri::command]
pub fn stats_focus(
    from: i64,
    to: i64,
    state: State<AppState>,
) -> Vec<crate::stats::FocusSession> {
    let segs = state.store.list_segments(from, to);
    let gap = state
        .store
        .get_setting("idle_threshold_ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(90000);
    crate::stats::focus_sessions(&segs, gap)
}

#[tauri::command]
pub fn stats_wordcloud(
    from: i64,
    to: i64,
    state: State<AppState>,
) -> Vec<crate::stats::WordFreq> {
    let segs = state.store.list_segments(from, to);
    crate::stats::word_cloud(&segs)
}

#[tauri::command]
pub fn delete_report(id: i64, state: State<AppState>) -> Result<(), String> {
    state.store.delete_report(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_reports(keep: Option<i64>, state: State<AppState>) -> Result<usize, String> {
    state.store.clear_reports(keep.unwrap_or(0)).map_err(|e| e.to_string())
}
