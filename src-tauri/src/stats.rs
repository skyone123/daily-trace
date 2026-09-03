use crate::store::WorkSegment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct HeatCell {
    pub day: u32,
    pub hour: u32,
    pub ms: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FocusSession {
    pub ts_start: i64,
    pub ts_end: i64,
    pub ms: i64,
    pub seg_count: i32,
    pub apps: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WordFreq {
    pub word: String,
    pub count: usize,
}

fn title_from_summary(s: &str) -> Option<String> {
    s.split_once('「')
        .and_then(|(_, a)| a.split_once('」').map(|(t, _)| t.to_string()))
}

/// 按星期(0=周一)×小时(0-23) 聚合工作时长
pub fn heatmap(segments: &[WorkSegment]) -> Vec<HeatCell> {
    use chrono::{Datelike, TimeZone, Timelike};

    let mut map: HashMap<(u32, u32), i64> = HashMap::new();
    for s in segments {
        if s.ts_end <= s.ts_start {
            continue;
        }
        let mut cur = s.ts_start;
        while cur < s.ts_end {
            let dt = match chrono::Local.timestamp_millis_opt(cur).single() {
                Some(d) => d,
                None => break,
            };
            let day = dt.weekday().num_days_from_monday() as u32;
            let hour = dt.hour() as u32;
            let next_hour_ndt = dt
                .naive_local()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
                + chrono::Duration::hours(1);
            let next_hour_ts = match chrono::Local
                .from_local_datetime(&next_hour_ndt)
                .single()
            {
                Some(d) => d.timestamp_millis(),
                None => cur + 3_600_000,
            };
            let end_in_hour = s.ts_end.min(next_hour_ts);
            *map.entry((day, hour)).or_insert(0) += end_in_hour - cur;
            cur = end_in_hour;
        }
    }
    let mut out: Vec<HeatCell> = map
        .into_iter()
        .map(|((day, hour), ms)| HeatCell { day, hour, ms })
        .collect();
    out.sort_by_key(|c| (c.day, c.hour));
    out
}

/// 将间隔小于阈值的连续段合并为专注会话，按时长降序
pub fn focus_sessions(segments: &[WorkSegment], idle_threshold_ms: i64) -> Vec<FocusSession> {
    let mut segs: Vec<&WorkSegment> = segments.iter().collect();
    segs.sort_by_key(|s| s.ts_start);

    let mut out: Vec<FocusSession> = Vec::new();
    let mut cur: Option<FocusSession> = None;

    let push_app = |v: &mut Vec<String>, app: &str| {
        if !app.is_empty() && !v.iter().any(|x| x == app) {
            v.push(app.to_string());
        }
    };

    for s in &segs {
        let app = s.category.clone().unwrap_or_default();
        match &mut cur {
            None => {
                cur = Some(FocusSession {
                    ts_start: s.ts_start,
                    ts_end: s.ts_end,
                    ms: s.ts_end - s.ts_start,
                    seg_count: 1,
                    apps: vec![],
                });
                if let Some(c) = cur.as_mut() {
                    push_app(&mut c.apps, &app);
                }
            }
            Some(c) => {
                let gap = s.ts_start.saturating_sub(c.ts_end);
                if gap > idle_threshold_ms {
                    out.push(c.clone());
                    cur = Some(FocusSession {
                        ts_start: s.ts_start,
                        ts_end: s.ts_end,
                        ms: s.ts_end - s.ts_start,
                        seg_count: 1,
                        apps: vec![],
                    });
                    if let Some(c) = cur.as_mut() {
                        push_app(&mut c.apps, &app);
                    }
                } else {
                    c.ts_end = s.ts_end;
                    c.ms = c.ts_end - c.ts_start;
                    c.seg_count += 1;
                    push_app(&mut c.apps, &app);
                }
            }
        }
    }
    if let Some(c) = cur {
        out.push(c);
    }
    out.sort_by(|a, b| b.ms.cmp(&a.ms));
    out
}

/// 从工作段标题提取高频词
pub fn word_cloud(segments: &[WorkSegment]) -> Vec<WordFreq> {
    let stop: &[&str] = &[
        "-", "—", "的", "和", "与", "及", "或", "在", "了", "是", ":", "：", "|", "·", "v", "v2",
        "com", "www", "http", "https", "cn", "pdf", "doc",
    ];
    let mut freq: HashMap<String, usize> = HashMap::new();

    let titles: Vec<String> = segments
        .iter()
        .filter_map(|s| s.summary.as_ref().and_then(|x| title_from_summary(x)))
        .collect();

    for t in &titles {
        for w in t.split(|c: char| {
            c.is_whitespace()
                || c == '-'
                || c == '—'
                || c == '：'
                || c == ':'
                || c == '|'
                || c == '·'
                || c == '/'
                || c == '\\'
                || c == '('
                || c == ')'
                || c == '「'
                || c == '」'
                || c == '.'
                || c == '_'
                || c == '，'
                || c == '。'
        }) {
            let w = w.trim();
            if w.chars().count() < 2 {
                continue;
            }
            let lw = w.to_lowercase();
            if stop.contains(&lw.as_str()) {
                continue;
            }
            *freq.entry(lw).or_insert(0) += 1;
        }
    }
    let mut out: Vec<WordFreq> = freq
        .into_iter()
        .map(|(word, count)| WordFreq { word, count })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count));
    out.truncate(24);
    out
}
