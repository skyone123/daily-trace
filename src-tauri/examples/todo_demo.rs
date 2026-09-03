use chrono::Utc;
use daily_trace_lib::aggregator;
use daily_trace_lib::capture::{CaptureSource, MockSource};
use daily_trace_lib::llm::Provider;
use daily_trace_lib::todo;
use daily_trace_lib::store::Store;

#[tokio::main]
async fn main() {
    let path = std::env::temp_dir().join("daily-trace-todo-demo.db");
    let _ = std::fs::remove_file(&path);
    let store = Store::new(&path).expect("db");

    let now = Utc::now().timestamp_millis();
    let start = now - 6 * 3600 * 1000;
    let mock = MockSource::new();
    let mut ts = start;
    let mut c = 0;
    while ts < now && c < 12 {
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
        c += 1;
    }
    println!("[1] 采集 {} 个事件", c);

    let n = aggregator::aggregate_and_store(&store, start, now, 90000).unwrap();
    println!("[2] 聚合为 {} 个工作段", n);

    let provider = Provider::Mock;
    let todos = todo::generate_todos(&store, &provider, "day")
        .await
        .expect("gen todos");
    let open = todos.iter().filter(|t| t.status == "open").count();
    println!("[3] 提取待办 {} 条（其中开放 {} 条）：", todos.len(), open);
    for t in &todos {
        println!("     - [{}] {}", t.status, t.title);
    }

    let done = todo::evaluate_todos(&store, &provider)
        .await
        .expect("eval todos");
    println!("[4] 完成评估回环：基于最近 24 小时记录，{} 个待办被判定已推进完成", done);

    let after = store.list_todos(None, 100);
    let d = after.iter().filter(|t| t.status == "done").count();
    println!("[5] 最终状态：done={}, open={}", d, after.len() - d);
    println!("\n闭环验证：采集 → 聚合 → 提取待办 → 基于记录评估完成 ✓");

    let _ = std::fs::remove_file(&path);
}
