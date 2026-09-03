use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub const DEFAULT_DAILY_PROMPT: &str = r#"你是 Daily Trace，根据用户今天的工作时间线生成一份简洁、专业的工作日报。

要求：
1. 按工作主题归类，不要罗列原始事件
2. 每个主题写 1-3 句，体现做了什么、产出什么
3. 末尾用一两句话总结今日重点与明日可关注事项
4. 语气客观、可直接交付，不要寒暄
5. 用中文，使用 Markdown 无序列表

工作时间线数据：
{segments}

用户附加要求（可为空）：
{extra}"#;

pub fn init_db(path: &std::path::Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        r#"
CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT
);
CREATE TABLE IF NOT EXISTS timeline_events (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  ts_start     INTEGER NOT NULL,
  ts_end       INTEGER,
  kind         TEXT NOT NULL,
  app_name     TEXT,
  window_title TEXT,
  content_ref  TEXT,
  content_hash TEXT,
  raw_meta     TEXT,
  created_at   INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_events_start ON timeline_events(ts_start);

CREATE TABLE IF NOT EXISTS work_segments (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  ts_start    INTEGER NOT NULL,
  ts_end      INTEGER NOT NULL,
  event_ids   TEXT NOT NULL,
  summary     TEXT,
  category    TEXT,
  tags        TEXT,
  focus_score REAL,
  ai_model    TEXT,
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_seg_start ON work_segments(ts_start);

CREATE TABLE IF NOT EXISTS reports (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  period       TEXT NOT NULL,
  period_start INTEGER NOT NULL,
  period_end   INTEGER NOT NULL,
  template_id  INTEGER,
  content      TEXT NOT NULL,
  segment_ids  TEXT,
  model        TEXT,
  created_at   INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_reports_period ON reports(period, period_start);

CREATE TABLE IF NOT EXISTS memories (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  kind       TEXT NOT NULL,
  key        TEXT,
  value      TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS todos (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  title         TEXT NOT NULL,
  description   TEXT,
  assignee      TEXT,
  due_at        INTEGER,
  parent_id     INTEGER,
  status        TEXT NOT NULL,
  source_seg_id INTEGER,
  attachments   TEXT,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS report_templates (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  name       TEXT NOT NULL,
  prompt     TEXT NOT NULL,
  structure  TEXT,
  is_builtin INTEGER DEFAULT 0,
  created_at INTEGER NOT NULL
);
"#,
    )?;

    seed_defaults(&conn)?;
    Ok(conn)
}

fn seed_defaults(conn: &Connection) -> rusqlite::Result<()> {
    let now = chrono::Utc::now().timestamp_millis();

    let exists: i64 = conn
        .query_row("SELECT COUNT(*) FROM settings WHERE key='paused'", [], |r| r.get(0))?;
    if exists == 0 {
        conn.execute(
            "INSERT INTO settings(key,value) VALUES
              ('paused','false'),
              ('capture_interval_ms','1500'),
              ('idle_threshold_ms','90000'),
              ('screenshot_min_interval_ms','120000'),
              ('delete_screenshot_after','true'),
              ('capture_screenshot','false'),
              ('excluded_apps',''),
              ('display_id','0'),
              ('model_kind','mock'),
              ('model_base_url','https://api.openai.com/v1'),
              ('model_api_key',''),
              ('model_name','gpt-4o-mini')",
            [],
        )?;
    }

    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM report_templates WHERE is_builtin=1",
        [],
        |r| r.get(0),
    )?;
    if exists == 0 {
        conn.execute(
            "INSERT INTO report_templates(name,prompt,structure,is_builtin,created_at) VALUES(?,?,?,?,?)",
            params!["日报", DEFAULT_DAILY_PROMPT, "[]", 1, now],
        )?;
    }

    let exists: i64 =
        conn.query_row("SELECT COUNT(*) FROM memories WHERE kind='category'", [], |r| {
            r.get(0)
        })?;
    if exists == 0 {
        let cats = [
            "研发编码",
            "产品与设计",
            "沟通协作",
            "会议",
            "文档与汇报",
            "学习与调研",
            "运维与杂务",
            "休息",
        ];
        for c in cats {
            conn.execute(
                "INSERT INTO memories(kind,key,value,created_at,updated_at) VALUES('category',?,?,'1','1')",
                params![c, c],
            )?;
        }
        let now2 = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "UPDATE memories SET created_at=?, updated_at=? WHERE created_at=1",
            params![now2, now2],
        )?;
    }

    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub id: i64,
    pub ts_start: i64,
    pub ts_end: Option<i64>,
    pub kind: String,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub content_ref: Option<String>,
    pub raw_meta: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkSegment {
    pub id: i64,
    pub ts_start: i64,
    pub ts_end: i64,
    pub event_ids: String,
    pub summary: Option<String>,
    pub category: Option<String>,
    pub tags: Option<String>,
    pub focus_score: Option<f64>,
    pub ai_model: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: i64,
    pub period: String,
    pub period_start: i64,
    pub period_end: i64,
    pub content: String,
    pub model: Option<String>,
    pub created_at: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub due_at: Option<i64>,
    pub parent_id: Option<i64>,
    pub status: String,
    pub source_seg_id: Option<i64>,
    pub attachments: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct Store {
    pub conn: Mutex<Connection>,
}

impl Store {
    pub fn new(path: &std::path::Path) -> rusqlite::Result<Self> {
        let conn = init_db(path)?;
        Ok(Store {
            conn: Mutex::new(conn),
        })
    }

    pub fn get_setting(&self, key: &str) -> Option<String> {
        let conn = self.conn.lock().ok()?;
        conn.query_row(
            "SELECT value FROM settings WHERE key=?",
            params![key],
            |r| r.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
    }

    pub fn set_setting(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO settings(key,value) VALUES(?,?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn insert_event(
        &self,
        ts_start: i64,
        ts_end: Option<i64>,
        kind: &str,
        app_name: Option<&str>,
        window_title: Option<&str>,
        content_ref: Option<&str>,
        raw_meta: Option<&str>,
    ) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO timeline_events(ts_start,ts_end,kind,app_name,window_title,content_ref,raw_meta,created_at) VALUES(?,?,?,?,?,?,?,?)",
            params![ts_start, ts_end, kind, app_name, window_title, content_ref, raw_meta, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn close_last_open_event(&self, ts_end: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE timeline_events SET ts_end=? WHERE ts_end IS NULL AND kind='app'",
            params![ts_end],
        )?;
        Ok(())
    }

    pub fn close_event(&self, id: i64, ts_end: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE timeline_events SET ts_end=? WHERE id=? AND ts_end IS NULL",
            params![ts_end, id],
        )?;
        Ok(())
    }

    pub fn list_events(&self, from: i64, to: i64, limit: i64) -> Vec<TimelineEvent> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = conn
            .prepare(
                "SELECT id,ts_start,ts_end,kind,app_name,window_title,content_ref,raw_meta
                 FROM timeline_events WHERE ts_start>=? AND ts_start<? ORDER BY ts_start ASC LIMIT ?",
            )
            .unwrap();
        let rows = stmt
            .query_map(params![from, to, limit], |r| {
                Ok(TimelineEvent {
                    id: r.get(0)?,
                    ts_start: r.get(1)?,
                    ts_end: r.get(2)?,
                    kind: r.get(3)?,
                    app_name: r.get(4)?,
                    window_title: r.get(5)?,
                    content_ref: r.get(6)?,
                    raw_meta: r.get(7)?,
                })
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn list_segments(&self, from: i64, to: i64) -> Vec<WorkSegment> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = conn
            .prepare(
                "SELECT id,ts_start,ts_end,event_ids,summary,category,tags,focus_score,ai_model
                 FROM work_segments WHERE ts_start>=? AND ts_start<? ORDER BY ts_start ASC",
            )
            .unwrap();
        let rows = stmt
            .query_map(params![from, to], |r| {
                Ok(WorkSegment {
                    id: r.get(0)?,
                    ts_start: r.get(1)?,
                    ts_end: r.get(2)?,
                    event_ids: r.get(3)?,
                    summary: r.get(4)?,
                    category: r.get(5)?,
                    tags: r.get(6)?,
                    focus_score: r.get(7)?,
                    ai_model: r.get(8)?,
                })
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn insert_segment(
        &self,
        ts_start: i64,
        ts_end: i64,
        event_ids: &str,
        summary: Option<&str>,
        category: Option<&str>,
        focus_score: Option<f64>,
        ai_model: Option<&str>,
    ) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO work_segments(ts_start,ts_end,event_ids,summary,category,tags,focus_score,ai_model,created_at,updated_at)
             VALUES(?,?,?,?,?,NULL,?,?,?,?)",
            params![ts_start, ts_end, event_ids, summary, category, focus_score, ai_model, now, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_unaggregated_events(&self, before_ts: i64) -> Vec<TimelineEvent> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = conn.prepare(
            "SELECT id,ts_start,ts_end,kind,app_name,window_title,content_ref,raw_meta
             FROM timeline_events WHERE ts_start<? AND ts_end IS NOT NULL
             ORDER BY ts_start ASC"
        ).unwrap();
        let rows = stmt
            .query_map(params![before_ts], |r| {
                Ok(TimelineEvent {
                    id: r.get(0)?,
                    ts_start: r.get(1)?,
                    ts_end: r.get(2)?,
                    kind: r.get(3)?,
                    app_name: r.get(4)?,
                    window_title: r.get(5)?,
                    content_ref: r.get(6)?,
                    raw_meta: r.get(7)?,
                })
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn mark_events_aggregated(&self, event_ids: &[i64]) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        let ids_json = serde_json::to_string(event_ids).unwrap_or_default();
        conn.execute(
            "UPDATE timeline_events SET content_ref = COALESCE(content_ref,'') 
             WHERE id IN (SELECT value FROM json_each(?))",
            params![ids_json],
        )?;
        Ok(())
    }

    pub fn insert_report(
        &self,
        period: &str,
        period_start: i64,
        period_end: i64,
        template_id: Option<i64>,
        content: &str,
        segment_ids: &str,
        model: &str,
    ) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO reports(period,period_start,period_end,template_id,content,segment_ids,model,created_at)
             VALUES(?,?,?,?,?,?,?,?)",
            params![period, period_start, period_end, template_id, content, segment_ids, model, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_reports(&self, limit: i64) -> Vec<Report> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = conn
            .prepare(
                "SELECT id,period,period_start,period_end,content,model,created_at
                 FROM reports ORDER BY created_at DESC LIMIT ?",
            )
            .unwrap();
        let rows = stmt
            .query_map(params![limit], |r| {
                Ok(Report {
                    id: r.get(0)?,
                    period: r.get(1)?,
                    period_start: r.get(2)?,
                    period_end: r.get(3)?,
                    content: r.get(4)?,
                    model: r.get::<_, Option<String>>(5)?,
                    created_at: r.get(6)?,
                })
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn delete_report(&self, id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM reports WHERE id=?", params![id])?;
        Ok(())
    }

    pub fn clear_reports(&self, keep: i64) -> rusqlite::Result<usize> {
        let conn = self.conn.lock().unwrap();
        let n = conn.execute(
            "DELETE FROM reports WHERE id NOT IN (SELECT id FROM reports ORDER BY created_at DESC LIMIT ?)",
            params![keep],
        )?;
        Ok(n)
    }

    pub fn get_template_prompt(&self, template_id: i64) -> Option<String> {
        let conn = self.conn.lock().ok()?;
        conn.query_row(
            "SELECT prompt FROM report_templates WHERE id=?",
            params![template_id],
            |r| r.get::<_, String>(0),
        )
        .ok()
    }

    pub fn list_categories(&self) -> Vec<String> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = conn
            .prepare("SELECT value FROM memories WHERE kind='category' ORDER BY id")
            .unwrap();
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn list_all_settings(&self) -> std::collections::HashMap<String, String> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return std::collections::HashMap::new(),
        };
        let mut stmt = conn.prepare("SELECT key,value FROM settings").unwrap();
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default()))
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn stats_by_app(&self, from: i64, to: i64) -> Vec<(String, i64)> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = conn.prepare(
            "SELECT COALESCE(app_name,'未知'), SUM(ts_end-ts_start) 
             FROM timeline_events WHERE ts_start>=? AND ts_start<? AND ts_end IS NOT NULL AND kind='app'
             GROUP BY app_name ORDER BY 2 DESC"
        ).unwrap();
        let rows = stmt
            .query_map(params![from, to], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn list_todos(&self, status: Option<&str>, limit: i64) -> Vec<Todo> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let base = "SELECT id,title,description,assignee,due_at,parent_id,status,source_seg_id,attachments,created_at,updated_at FROM todos";
        let mapper = |r: &rusqlite::Row| -> rusqlite::Result<Todo> {
            Ok(Todo {
                id: r.get(0)?,
                title: r.get(1)?,
                description: r.get(2)?,
                assignee: r.get(3)?,
                due_at: r.get(4)?,
                parent_id: r.get(5)?,
                status: r.get(6)?,
                source_seg_id: r.get(7)?,
                attachments: r.get(8)?,
                created_at: r.get(9)?,
                updated_at: r.get(10)?,
            })
        };
        match status {
            Some(st) => {
                if let Ok(mut stmt) =
                    conn.prepare(&format!("{} WHERE status=? ORDER BY updated_at DESC LIMIT ?", base))
                {
                    let rows = stmt.query_map(params![st, limit], mapper).unwrap();
                    rows.filter_map(|r| r.ok()).collect()
                } else {
                    vec![]
                }
            }
            None => {
                if let Ok(mut stmt) =
                    conn.prepare(&format!("{} ORDER BY updated_at DESC LIMIT ?", base))
                {
                    let rows = stmt.query_map(params![limit], mapper).unwrap();
                    rows.filter_map(|r| r.ok()).collect()
                } else {
                    vec![]
                }
            }
        }
    }

    pub fn insert_todo(
        &self,
        title: &str,
        description: Option<&str>,
        assignee: Option<&str>,
        due_at: Option<i64>,
        source_seg_id: Option<i64>,
    ) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO todos(title,description,assignee,due_at,parent_id,status,source_seg_id,attachments,created_at,updated_at) VALUES(?,?,?,?,NULL,'open',?,NULL,?,?)",
            params![title, description, assignee, due_at, source_seg_id, now, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_todo_status(&self, id: i64, status: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "UPDATE todos SET status=?, updated_at=? WHERE id=?",
            params![status, now, id],
        )?;
        Ok(())
    }

    pub fn update_segment_category(&self, id: i64, category: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "UPDATE work_segments SET category=?, updated_at=? WHERE id=?",
            params![category, now, id],
        )?;
        Ok(())
    }

    pub fn list_memories(&self) -> Vec<(String, String, String)> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return vec![],
        };
        let mut stmt = match conn.prepare("SELECT kind,key,value FROM memories ORDER BY id") {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                ))
            })
            .unwrap();
        rows.filter_map(|r| r.ok()).collect()
    }

    pub fn export_data(&self) -> serde_json::Value {
        serde_json::json!({
            "version": 1,
            "exported_at": chrono::Utc::now().timestamp_millis(),
            "settings": self.list_all_settings(),
            "memories": self.list_memories(),
            "events": self.list_events(0, i64::MAX, 100000),
            "segments": self.list_segments(0, i64::MAX),
            "reports": self.list_reports(100000),
            "todos": self.list_todos(None, 100000),
        })
    }
}
