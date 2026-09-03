use crate::store::{Store, TimelineEvent, WorkSegment};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SegmentDraft {
    pub ts_start: i64,
    pub ts_end: i64,
    pub event_ids: Vec<i64>,
    pub app_name: String,
    pub titles: Vec<String>,
    pub duration_ms: i64,
}

pub fn aggregate(events: Vec<TimelineEvent>, gap_threshold_ms: i64) -> Vec<SegmentDraft> {
    let mut drafts: Vec<SegmentDraft> = Vec::new();
    let mut current: Option<SegmentDraft> = None;
    let mut last_end: i64 = 0;

    for e in events.iter() {
        let ts_end = e.ts_end.unwrap_or(e.ts_start);
        let app = e.app_name.clone().unwrap_or_else(|| "未知".to_string());
        let title = e.window_title.clone().unwrap_or_default();

        let start_new = match &current {
            None => true,
            Some(c) => {
                let gap = e.ts_start.saturating_sub(last_end);
                c.app_name != app || gap > gap_threshold_ms
            }
        };

        if start_new {
            if let Some(c) = current.take() {
                if c.duration_ms > 0 {
                    drafts.push(c);
                }
            }
            current = Some(SegmentDraft {
                ts_start: e.ts_start,
                ts_end,
                event_ids: vec![e.id],
                app_name: app,
                titles: if title.is_empty() { vec![] } else { vec![title] },
                duration_ms: ts_end.saturating_sub(e.ts_start),
            });
        } else if let Some(c) = current.as_mut() {
            c.event_ids.push(e.id);
            c.ts_end = ts_end;
            c.duration_ms = c.ts_end.saturating_sub(c.ts_start);
            if !title.is_empty() {
                c.titles.push(title);
            }
        }
        last_end = ts_end;
    }
    if let Some(c) = current {
        if c.duration_ms > 0 {
            drafts.push(c);
        }
    }
    drafts
}

pub fn draft_summary(d: &SegmentDraft) -> String {
    let mins = (d.duration_ms / 60000).max(1);
    let mut s = format!("在 {} 工作 {} 分钟", d.app_name, mins);
    if !d.titles.is_empty() {
        let t = d.titles.first().unwrap();
        let t = t.chars().take(40).collect::<String>();
        s.push_str(&format!("，涉及「{}」", t));
        if d.titles.len() > 1 {
            s.push_str(&format!(" 等 {} 个窗口", d.titles.len()));
        }
    }
    s
}

pub fn aggregate_and_store(
    store: &Store,
    from: i64,
    to: i64,
    gap_threshold_ms: i64,
) -> Result<usize, Box<dyn std::error::Error>> {
    let conn = store.conn.lock().unwrap();
    conn.execute(
        "DELETE FROM work_segments WHERE ts_start>=? AND ts_start<?",
        rusqlite::params![from, to],
    )?;
    drop(conn);

    let events = store.list_events(from, to, 10000);
    let drafts = aggregate(events, gap_threshold_ms);
    let n = drafts.len();
    for d in drafts.iter() {
        let ids_json = serde_json::to_string(&d.event_ids).unwrap_or_default();
        let summary = draft_summary(d);
        let _ = store.insert_segment(
            d.ts_start,
            d.ts_end,
            &ids_json,
            Some(&summary),
            Some(&d.app_name),
            None,
            None,
        );
    }
    Ok(n)
}

#[allow(dead_code)]
pub fn list_segments(store: &Store, from: i64, to: i64) -> Vec<WorkSegment> {
    store.list_segments(from, to)
}
