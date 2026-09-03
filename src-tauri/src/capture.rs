use crate::store::Store;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Activity {
    pub app_name: String,
    pub window_title: String,
}

pub trait CaptureSource: Send + Sync {
    fn current_activity(&self) -> Option<Activity>;
}

pub struct WindowsSource;

impl CaptureSource for WindowsSource {
    #[cfg(windows)]
    fn current_activity(&self) -> Option<Activity> {
        capture_windows_foreground()
    }
    #[cfg(not(windows))]
    fn current_activity(&self) -> Option<Activity> {
        None
    }
}

pub struct MockSource {
    counter: std::sync::atomic::AtomicUsize,
}

impl MockSource {
    pub fn new() -> Self {
        Self {
            counter: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl CaptureSource for MockSource {
    fn current_activity(&self) -> Option<Activity> {
        let apps = [
            ("VS Code", "main.rs - daily-trace"),
            ("Chrome", "Daily Trace - 官网"),
            ("微信", "工作群"),
            ("Notion", "需求文档"),
            ("Terminal", "cargo build"),
            ("Figma", "首页设计稿"),
        ];
        let c = self.counter.fetch_add(1, Ordering::Relaxed);
        let (app, title) = apps[c % apps.len()];
        Some(Activity {
            app_name: app.to_string(),
            window_title: title.to_string(),
        })
    }
}

struct Inner {
    last_app: Option<String>,
    last_title: Option<String>,
    open_event_id: Option<i64>,
    open_since: i64,
}

pub struct Collector {
    store: Arc<Store>,
    source: Box<dyn CaptureSource>,
    inner: Mutex<Inner>,
    pub paused: AtomicBool,
}

impl Collector {
    pub fn new(store: Arc<Store>, source: Box<dyn CaptureSource>, paused: bool) -> Self {
        Collector {
            store,
            source,
            inner: Mutex::new(Inner {
                last_app: None,
                last_title: None,
                open_event_id: None,
                open_since: 0,
            }),
            paused: AtomicBool::new(paused),
        }
    }

    pub async fn run(self: Arc<Self>) {
        let interval_ms: u64 = self
            .store
            .get_setting("capture_interval_ms")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1500);
        let mut tk = interval(Duration::from_millis(interval_ms));
        loop {
            tk.tick().await;
            let _ = self.tick();
        }
    }

    fn tick(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.paused.load(Ordering::Relaxed) {
            return Ok(());
        }
        let now = chrono::Utc::now().timestamp_millis();
        let activity = match self.source.current_activity() {
            Some(a) => a,
            None => return Ok(()),
        };

        if is_excluded(&self.store, &activity.app_name, &activity.window_title) {
            return Ok(());
        }

        let mut inner = self.inner.lock().unwrap();
        let changed = inner.last_app.as_deref() != Some(activity.app_name.as_str())
            || inner.last_title.as_deref() != Some(activity.window_title.as_str());

        if changed {
            if let Some(id) = inner.open_event_id {
                let _ = self.store.close_event(id, now);
            }
            let id = self.store.insert_event(
                now,
                None,
                "app",
                Some(&activity.app_name),
                Some(&activity.window_title),
                None,
                None,
            )?;
            inner.open_event_id = Some(id);
            inner.open_since = now;
            inner.last_app = Some(activity.app_name.clone());
            inner.last_title = Some(activity.window_title.clone());
        }
        Ok(())
    }
}

fn is_excluded(store: &Store, app: &str, title: &str) -> bool {
    let raw = store.get_setting("excluded_apps").unwrap_or_default();
    if raw.is_empty() {
        return false;
    }
    raw.split(',')
        .map(|s| s.trim().to_lowercase())
        .any(|pat| {
            !pat.is_empty() && (app.to_lowercase().contains(&pat) || title.to_lowercase().contains(&pat))
        })
}

#[cfg(windows)]
fn capture_windows_foreground() -> Option<Activity> {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        let title = if len > 0 {
            String::from_utf16_lossy(&title_buf[..len as usize])
        } else {
            String::new()
        };

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32));
        if pid == 0 {
            return Some(Activity {
                app_name: "未知".to_string(),
                window_title: title,
            });
        }

        let app_name = get_process_name(pid).unwrap_or_else(|| "未知进程".to_string());

        Some(Activity {
            app_name,
            window_title: title,
        })
    }
}

#[cfg(windows)]
fn get_process_name(pid: u32) -> Option<String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::core::PWSTR;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ptr = PWSTR::from_raw(buf.as_mut_ptr());
        let ok = QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), ptr, &mut len);
        let _ = CloseHandle(handle);
        if !ok.is_ok() {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        std::path::Path::new(&path)
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
fn get_process_name(_pid: u32) -> Option<String> {
    None
}
