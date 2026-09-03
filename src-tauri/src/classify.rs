use crate::llm::Provider;
use crate::store::{Store, WorkSegment};

/// Mock 分类：按 app 名 + 摘要关键词映射到 memories 分类体系
fn mock_classify(app: &str, summary: &str) -> &'static str {
    let s = format!("{} {}", app, summary).to_lowercase();
    if s.contains("figma") || s.contains("设计") {
        "产品与设计"
    } else if s.contains("微信") || s.contains("群") || s.contains("slack") || s.contains("飞书") {
        "沟通协作"
    } else if s.contains("notion") || s.contains("文档") || s.contains("汇报") {
        "文档与汇报"
    } else if s.contains("chrome") || s.contains("调研") || s.contains("文档") || s.contains("学习") {
        "学习与调研"
    } else if s.contains("code") || s.contains("cargo") || s.contains("terminal") || s.contains(".rs")
        || s.contains("main.rs") || s.contains("lib.rs") || s.contains("vs code")
    {
        "研发编码"
    } else if s.contains("运维") || s.contains("部署") || s.contains("build") {
        "运维与杂务"
    } else if s.contains("休息") || s.contains("bilibili") || s.contains("youtube") {
        "休息"
    } else {
        "运维与杂务"
    }
}

/// 对 [from, to) 内的工作段做 AI 分类，更新 segment.category
pub async fn classify_segments(
    store: &Store,
    provider: &Provider,
    from: i64,
    to: i64,
) -> Result<usize, String> {
    let cats = store.list_categories();
    if cats.is_empty() {
        return Err("未配置分类".into());
    }
    let segments = store.list_segments(from, to);
    if segments.is_empty() {
        return Ok(0);
    }
    let mut n = 0;
    for s in segments.iter() {
        let app = s.category.clone().unwrap_or_default();
        let summary = s.summary.clone().unwrap_or_default();
        let category: String = match provider {
            Provider::Mock => mock_classify(&app, &summary).to_string(),
            Provider::OpenAI { .. } => {
                let system = "你是工作分类助手。从给定分类列表中为工作段选一个最贴切的分类。只返回分类名，不要解释、不要标点。";
                let user = format!(
                    "分类列表：{}\n\n工作段：{}\n（原始应用：{}）\n请选一个分类。",
                    cats.join("、"),
                    summary,
                    app
                );
                let (out, _) = provider
                    .chat(system, &user)
                    .await
                    .map_err(|e| e.to_string())?;
                out.trim().lines().next().unwrap_or("运维与杂务").trim().to_string()
            }
        };
        // 校验分类在列表内，否则兜底
        let final_cat = if cats.iter().any(|c| c == &category) {
            category
        } else {
            mock_classify(&app, &summary).to_string()
        };
        store
            .update_segment_category(s.id, &final_cat)
            .map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(n)
}

#[allow(dead_code)]
pub fn _unused(_s: &WorkSegment) {}
