use chrono::Utc;
use std::sync::atomic::AtomicUsize;
use daily_trace_lib::aggregator;
use daily_trace_lib::capture::{CaptureSource, MockSource};
use daily_trace_lib::llm::Provider;
use daily_trace_lib::report;
use daily_trace_lib::store::Store;

fn main() {
    let path = std::env::temp_dir().join("daily-trace-cli-demo.db");
    let _ = std::fs::remove_file(&path);
    let store = Store::new(&path).expect("db");

    let now = Utc::now().timestamp_millis();
    let day_start = now - 8 * 3600 * 1000;
    let mock = MockSource::new();
    let mut ts = day_start;
    let mut count = 0;
    while ts < now && count < 22 {
        let act = mock.current_activity().unwrap();
        let dur = (20 + (ts / 60000) % 30) * 60 * 1000;
        let end = (ts + dur).min(now);
        store
            .insert_event(
                ts,
                Some(end),
                "app",
                Some(&act.app_name),
                Some(&act.window_title),
                None,
                None,
            )
            .unwrap();
        ts = end + 5 * 60 * 1000;
        count += 1;
    }
    println!("[1/3] 采集完成：写入 {} 个原始事件", count);

    let n = aggregator::aggregate_and_store(&store, day_start, now, 90000).unwrap();
    let segs = store.list_segments(day_start, now);
    println!("[2/3] 聚合完成：{} 个工作段", n);
    for s in segs.iter().take(6) {
        let mins = (s.ts_end - s.ts_start) / 60000;
        println!(
            "       - {} | {} | {}分钟",
            s.category.as_deref().unwrap_or("?"),
            s.summary.as_deref().unwrap_or(""),
            mins
        );
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let rep = rt
        .block_on(async {
            let provider = Provider::Mock;
            report::generate_report(&store, &provider, "day", "").await
        })
        .expect("report");
    println!("\n[3/3] 日报生成完成（模型: {}）\n", rep.model.unwrap_or_default());
    println!("{}", rep.content);

    let _ = std::fs::remove_file(&path);
    let _ = AtomicUsize::new(0);
}
