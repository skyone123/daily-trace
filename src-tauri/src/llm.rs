use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("网络请求失败: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("API 返回错误: status={0} body={1}")]
    Api(u16, String),
    #[error("解析响应失败: {0}")]
    Parse(String),
    #[error("未配置 API Key，无法调用云模型")]
    NoApiKey,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone)]
pub enum Provider {
    Mock,
    OpenAI {
        base_url: String,
        api_key: String,
        model: String,
    },
}

#[derive(Serialize)]
struct OpenAIRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAIMessage<'a>>,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize)]
struct OpenAIMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIRespMessage,
}

#[derive(Deserialize)]
struct OpenAIRespMessage {
    content: String,
}

impl Provider {
    pub async fn chat(
        &self,
        system: &str,
        user: &str,
    ) -> Result<(String, String), LlmError> {
        match self {
            Provider::Mock => {
                let out = mock_daily_report(user);
                Ok((out, "mock-local".to_string()))
            }
            Provider::OpenAI {
                base_url,
                api_key,
                model,
            } => {
                if api_key.is_empty() {
                    return Err(LlmError::NoApiKey);
                }
                let client = Client::builder()
                    .timeout(std::time::Duration::from_secs(60))
                    .build()?;
                let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
                let body = OpenAIRequest {
                    model,
                    messages: vec![
                        OpenAIMessage {
                            role: "system",
                            content: system,
                        },
                        OpenAIMessage {
                            role: "user",
                            content: user,
                        },
                    ],
                    temperature: 0.4,
                    stream: false,
                };
                let resp = client
                    .post(&url)
                    .bearer_auth(api_key)
                    .json(&body)
                    .send()
                    .await?;
                let status = resp.status().as_u16();
                if !resp.status().is_success() {
                    let text = resp.text().await.unwrap_or_default();
                    return Err(LlmError::Api(status, text));
                }
                let parsed: OpenAIResponse = resp
                    .json()
                    .await
                    .map_err(|e| LlmError::Parse(e.to_string()))?;
                let content = parsed
                    .choices
                    .into_iter()
                    .next()
                    .map(|c| c.message.content)
                    .ok_or_else(|| LlmError::Parse("响应缺少 choices".into()))?;
                Ok((content, model.clone()))
            }
        }
    }
}

fn mock_daily_report(segments_text: &str) -> String {
    use std::collections::BTreeMap;

    let mut by_app: BTreeMap<String, (i64, Vec<String>)> = BTreeMap::new();
    let mut total = 0i64;
    let mut count = 0usize;

    for line in segments_text.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('[') {
            continue;
        }
        if let Some(end_bracket) = line.find(']') {
            let _time_range = &line[1..end_bracket];
            let rest = &line[end_bracket + 1..].trim_start();
            let app = rest.split('|').next().unwrap_or("未知").trim();
            let minutes = if let Some(p) = rest.rfind('(') {
                rest[p + 1..]
                    .trim_end_matches("分钟)")
                    .trim_end_matches("分)")
                    .parse::<i64>()
                    .unwrap_or(0)
            } else {
                0
            };
            let title = rest
                .split_once('「')
                .and_then(|(_, after)| after.split_once('」').map(|(t, _)| t.to_string()))
                .unwrap_or_default();
            let entry = by_app.entry(app.to_string()).or_insert((0, vec![]));
            entry.0 += minutes;
            if !title.is_empty() {
                entry.1.push(title.clone());
            }
            total += minutes;
            count += 1;
        }
    }

    let mut out = String::new();
    out.push_str("## 今日工作日报\n\n");
    if by_app.is_empty() {
        out.push_str("_今日暂未采集到工作记录。_\n");
        return out;
    }
    for (app, (mins, titles)) in by_app.iter() {
        out.push_str(&format!("### {}（{}分钟）\n", app, mins));
        if titles.is_empty() {
            out.push_str("- 专注处理相关事务\n");
        } else {
            for t in titles.iter().take(3) {
                out.push_str(&format!("- 涉及「{}」\n", t));
            }
        }
        out.push('\n');
    }
    let dur_str = if total < 60 {
        format!("{} 分钟", total)
    } else {
        format!("{:.1} 小时", total as f64 / 60.0)
    };
    out.push_str(&format!(
        "本日记录工作段 {} 条，有效工作时长约 {}。",
        count, dur_str
    ));
    out.push_str(
        "\n\n> 当前为离线 Mock 模式生成的日报。在设置中配置云模型 API Key 后，将获得 AI 智能归类与润色。",
    );
    out
}
