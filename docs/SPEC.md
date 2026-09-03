# Daily Trace 复刻产品规格说明书 (SPEC)

> 版本：v0.2 (Decisions Locked)
> 日期：2026-09-02
> 状态：已基线，可进入实现

---

## 0. 文档目的

本文档定义一个"本地优先的 AI 工作记忆与复盘助手"的完整产品规格，作为复刻实现的唯一设计依据。
覆盖：产品定位、核心闭环、功能模块、系统架构、数据模型、采集策略、AI 理解流程、报告生成、隐私设计、开放 API、技术选型、MVP 范围与里程碑。

阅读对象：架构 / 全栈 / 算法 / 产品 / 测试。

---

## 1. 产品概述

### 1.1 一句话定位
自动采集工作轨迹，AI 理解分类，生成日报/周报/月报，提取并跟踪待办，把工作真正沉淀为成果的本地优先 AI 工作助手。

### 1.2 核心价值主张
- 不改变工作习惯，自动记录"今天做了什么"
- 一键生成可交付的报告，省去回忆与整理时间
- 从真实记录中提取下一步行动，并基于后续记录评估完成情况
- 数据本地优先，采集与上报边界由用户决定

### 1.3 目标用户
- 知识工作者（研发、产品、设计、运营、咨询等）
- 需要定期汇报（日报/周报/月报）的人群
- 希望复盘并推进下一步行动的个人与小团队

### 1.4 非目标（明确不做）
- 不是团队监控/员工行为审计工具
- 不是项目管理软件（不做甘特、不做协作流）
- 不做云端 SaaS 多租户托管（隐私优先，默认本地）

---

## 2. 核心闭环（第一设计原则）

```
┌───────────────────────────────────────────────────────┐
│                                                       │
│   1. 自动记录  ──>  2. AI 理解整理  ──>  3. 复盘汇报  │
│                                                       │
│                          ^                            │
│                          │                            │
│                          └── 4. 推进下一步 ────────────┤
│                                  │                    │
│                                  v                    │
│                     Agent 写回工作记录 (Lobster API)  │
└───────────────────────────────────────────────────────┘
```

| 阶段 | 输入 | 输出 |
|---|---|---|
| 自动记录 | 屏幕活动、应用、剪贴板、主动输入 | 原始事件流 |
| AI 理解整理 | 原始事件流 + 个人记忆 + 自定义分类 | 工作段、分类、摘要 |
| 复盘汇报 | 工作段 + 模板 + 用户要求 | 日/周/月报、可视化 |
| 推进下一步 | 工作段 + 待办池 | 待办、负责人、截止、完成评估 |

**闭环判据**：任意功能模块的设计必须能回答"它属于闭环哪一环，以及它是否让闭环更完整"。不能孤立存在。

---

## 3. 功能模块清单

| 模块 | 所属闭环环节 | MVP | 后续 |
|---|---|---|---|
| 自动采集（截图/应用/剪贴板/输入） | 记录 | v0.1 | - |
| 事件聚合（原始事件 → 工作段） | 记录 | v0.1 | - |
| 本地数据库与导出备份 | 记录 | v0.1 | - |
| AI 内容理解与分类 | 理解 | v0.1 | - |
| 个人记忆 / 自定义分类管理 | 理解 | v0.1 | - |
| 工作时间线 UI | 理解 | v0.1 | - |
| 日报/周报/月报生成 | 汇报 | v0.1 | - |
| 报告模板系统 + 模板市场 | 汇报 | v0.2 | - |
| 可视化报告 / 网页 AI 对话 | 汇报 | v0.3 | - |
| 应用统计 / 热力图 / 词云 / 专注时段 | 洞察 | v0.2 | - |
| 待办提取与跟踪 | 推进 | v0.2 | - |
| 待办完成评估（基于后续记录） | 推进 | v0.3 | - |
| Lobster 本地服务（开放 API） | 开放 | v0.3 | - |
| Agent 读写接入 | 开放 | v0.3 | - |
| 微信小程序 / 移动端 | 消费 | - | v1.0 |
| 隐私控制（暂停/排除/选择显示器/删图） | 全局 | v0.1 | - |
| 模型管理（云/自定义/本地/网页账号） | 理解 | v0.1 | - |
| 跨设备同步（可选） | 全局 | - | v1.0 |

---

## 4. 系统架构

### 4.1 分层架构

```
┌─────────────────────────────────────────────────────┐
│  UI 层 (Tauri Webview, React/Svelte + Tailwind)     │
│  时间线 / 报告 / 统计 / 待办 / 设置                  │
└───────────────────────┬─────────────────────────────┘
                        │ IPC / local HTTP
┌───────────────────────▼─────────────────────────────┐
│  应用核心层 (Rust)                                   │
│  采集调度 / 聚合引擎 / 报告编排 / 待办跟踪器          │
└──┬────────────────┬───────────────┬────────────────┘
   │                │               │
┌──▼──────┐   ┌─────▼─────┐  ┌──────▼──────────┐
│ 采集层  │   │ 存储层    │  │ AI 理解层         │
│ (Rust) │   │ (SQLite) │  │ (Provider 抽象)   │
└────────┘   └──────────┘  └──────────────────┘
   │
   ↓
┌──────────────────────────────────────────────────────┐
│  开放层 (Lobster 本地 HTTP 服务)                     │
│  GET /timeline  GET /report  POST /events  ...      │
└──────────────────────────────────────────────────────┘
```

### 4.2 进程模型

- **主进程（Tauri core）**：UI、报告编排、设置
- **采集守护进程**：独立常驻，保证记录连续性，UI 退出仍运行
- **Lobster 服务**：独立 HTTP 服务，可被外部 Agent 调用

守护进程与主进程通过本地 IPC 或共享 SQLite 解耦，保证"关掉 UI 也能记录"。

---

## 5. 数据模型

### 5.1 核心表（SQLite）

```sql
-- 原始采集事件
CREATE TABLE timeline_events (
  id            INTEGER PRIMARY KEY,
  ts_start      INTEGER NOT NULL,        -- epoch ms
  ts_end        INTEGER,
  kind          TEXT NOT NULL,           -- app|screenshot|clipboard|text|audio|video|manual
  app_name      TEXT,
  window_title  TEXT,
  display_id    INTEGER,
  content_ref   TEXT,                    -- 本地文件路径或内联文本
  content_hash  TEXT,                    -- 去重用
  raw_meta      TEXT,                    -- JSON
  created_at    INTEGER NOT NULL
);

-- AI 聚合后的工作段
CREATE TABLE work_segments (
  id            INTEGER PRIMARY KEY,
  ts_start      INTEGER NOT NULL,
  ts_end        INTEGER NOT NULL,
  event_ids     TEXT NOT NULL,           -- JSON array
  summary       TEXT,                     -- AI 生成摘要
  category      TEXT,                      -- 自定义分类
  tags          TEXT,                      -- JSON array
  focus_score   REAL,                      -- 专注度 0~1
  ai_model      TEXT,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL
);

-- 报告
CREATE TABLE reports (
  id            INTEGER PRIMARY KEY,
  period        TEXT NOT NULL,           -- day|week|month
  period_start  INTEGER NOT NULL,
  period_end    INTEGER NOT NULL,
  template_id   INTEGER,
  content       TEXT NOT NULL,           -- 渲染后内容 (markdown/html)
  segment_ids   TEXT,                    -- JSON array
  model         TEXT,
  created_at    INTEGER NOT NULL
);

-- 待办
CREATE TABLE todos (
  id            INTEGER PRIMARY KEY,
  title         TEXT NOT NULL,
  description   TEXT,
  assignee      TEXT,
  due_at        INTEGER,
  parent_id    INTEGER,                  -- 子任务
  status        TEXT NOT NULL,           -- open|doing|done|skipped
  source_seg_id INTEGER,                 -- 来源工作段
  attachments   TEXT,                     -- JSON
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL
);

-- 个人记忆 / 长期偏好
CREATE TABLE memories (
  id            INTEGER PRIMARY KEY,
  kind          TEXT NOT NULL,           -- category|preference|fact
  key           TEXT,
  value         TEXT,
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL
);

-- 报告模板
CREATE TABLE report_templates (
  id            INTEGER PRIMARY KEY,
  name          TEXT NOT NULL,
  prompt        TEXT NOT NULL,
  structure     TEXT,                     -- JSON 结构定义
  is_builtin    INTEGER DEFAULT 0,
  created_at    INTEGER NOT NULL
);
```

### 5.2 关键设计点
- 原始事件与聚合工作段分离，便于重新跑 AI 而不丢原始数据
- `content_ref` 指向本地文件，截图分析后可物理删除只留 hash
- 个人记忆独立成表，是分类与报告质量的关键

---

## 6. 采集策略

### 6.1 采集维度
| 维度 | 内容 | 触发 |
|---|---|---|
| 应用活动 | 进程名、窗口标题、停留时长 | 前台切换 / 定时（1s 轮询） |
| 截图 | 当前屏幕（可选显示器） | 事件触发（切换/空闲超 N 秒）+ 最小间隔 |
| 剪贴板 | 文本/图片 | 剪贴板变更事件 |
| 主动输入 | 文字/图片/音视频 | 用户手动添加 |
| 音视频 | 录音/录屏 | 用户主动 |

### 6.2 事件触发式采集（关键）
固定频率截图会爆炸 token 与存储。策略：
- 前台窗口切换 → 立即截一张
- 空闲超过阈值（如 90s）→ 不截，标记 idle
- 连续操作同一应用 → 最小间隔（如 120s）才再截
- 检测到内容变化（标题变化/剪贴板变化）→ 降级再截

### 6.3 平台采集 API
| 平台 | 应用活动 | 截图 |
|---|---|---|
| macOS | `CGWindowListCopyWindowInfo` / `NSWorkspace` | `ScreenCaptureKit` |
| Windows | `EnumWindows` / `GetForegroundWindow` / `GetWindowText` | `Windows.Graphics.Capture` |

### 6.4 事件聚合算法（原始 → 工作段）
1. 按 ts 排序原始事件
2. 同应用连续事件合并，时长累加
3. 空闲段（idle > 阈值）切断
4. 段内若有多张截图，取代表性 1~2 张送 AI
5. 输出 `work_segment`，含起止时间、应用、标题、代表截图

聚合在本地完成，**只有聚合后的工作段才送 AI**，原始事件留底。

---

## 7. AI 理解流程

### 7.1 Provider 抽象

```ts
interface LLMProvider {
  id: string;
  kind: "cloud" | "custom" | "local" | "web";
  chat(req: ChatRequest): Promise<ChatResponse>;
  // OpenAI 兼容协议，自定义 base_url + key
}
```

四类来源：
- **cloud**：默认云模型（如豆包/通义/GPT）
- **custom**：用户填 base_url + key（OpenAI 兼容）
- **local**：Ollama / llama.cpp 本地
- **web**：复用浏览器 Cookie 调网页账号（省 API 费）

### 7.2 理解任务链
```
工作段 ──> 内容识别 ──> 智能分类(用 memories) ──> 摘要 ──> 聚合到时间线
                                   │
                                   └── 待办提取(可选)
```

### 7.3 Prompt 设计原则
- 分类必须参考 `memories` 表中的用户自定义分类，禁止臆造
- 摘要要可读、可拼接到日报，不返回原始事件列表
- 待办提取要带上下文（来源段、时间），便于回溯
- 所有 prompt 模板化，放 `prompts/` 目录

---

## 8. 报告生成

### 8.1 生成流程
1. 选择周期（日/周/月）+ 模板
2. 拉取周期内 `work_segments`
3. 拼装 context：工作段摘要 + 应用统计 + 用户要求
4. 调 LLM 生成
5. 渲染（Markdown / 可视化图表）
6. 存 `reports`，支持复制/导出/继续修改

### 8.2 模板系统
- 内置模板：标准日报、周报、月报
- 自定义模板：用户可定义 prompt + 结构
- 模板市场（后续）：社区分享

### 8.3 可视化（v0.3）
- 时间线甘特
- 应用时长饼图
- 热力图（按小时/天）
- 词云
- 专注时段排行榜

---

## 9. 待办与完成评估

### 9.1 待办提取
- AI 从工作段摘要中提取候选待办
- 补充：负责人、截止时间、子任务、附件
- 来源 `source_seg_id` 保留可追溯

### 9.2 完成评估回环（闭环灵魂）
```
新工作段产生 ──> 匹配 open 待办 ──> AI 判断是否已完成 ──> 更新 status
```
- 匹配：标题/描述语义相似 + 时间窗口
- 评估：定期或用户触发
- 不做这一步，产品退化为"日报生成器"，必须实现

---

## 10. 隐私设计（必须复刻的卖点）

| 控制项 | 实现 |
|---|---|
| 暂停记录 | 全局开关，守护进程立即停止采集 |
| 排除敏感应用 | 黑名单（进程名/窗口标题正则匹配），命中则不采集 |
| 选择显示器 | 多屏时仅采集指定屏 |
| 关闭详细上下文 | 只记应用不记截图/内容 |
| 截图分析后删除 | AI 处理完即物理删图，只留 hash |
| 云模型/同步按需开启 | 默认全本地，开关显式 |
| 数据导出/备份/清理 | 本地数据库可导出/恢复/一键清空 |

**原则**：默认配置 = 最大隐私。任何上云能力默认关闭，由用户显式开启。

---

## 11. 开放层 - Lobster 本地服务

### 11.1 定位
本地起 HTTP 服务，让外部 Agent 与Daily Trace双向交互：
- 读：查时间线/报告/统计/待办
- 写：主动写入工作记录（闭环回写）

### 11.2 API 草案

```
GET  /api/timeline?from=&to=&limit=
GET  /api/reports/:id
GET  /api/stats/heatmap?from=&to=
GET  /api/todos?status=open
POST /api/events          # Agent 写入工作记录
POST /api/todos           # Agent 创建待办
PATCH /api/todos/:id      # Agent 更新待办
GET  /api/memories
POST /api/memories
```

### 11.3 安全
- 仅监听 127.0.0.1
- 首次启用生成 token，外部请求需带 Bearer
- 写操作需二次确认或白名单 Agent

---

## 12. 技术选型

| 层 | 选型 | 理由 |
|---|---|---|
| 桌面壳 | **Tauri** | 体积小、原生权限够采集、跨平台 |
| 核心语言 | Rust | 采集性能 + Tauri 原生 |
| UI | React + Tailwind | 生态成熟，可视化库多 |
| 存储 | SQLite | 本地优先，单文件易备份 |
| AI | OpenAI 兼容协议 + Ollama | 统一抽象，可插拔 |
| 本地服务 | axum (Rust) | 与主进程同语言 |
| 图表 | Recharts / ECharts | 热力图、饼图、甘特 |
| 小程序 | 后期 Taro/原生 | v1.0 才做 |

> Electron 也可行，但 Tauri 对"本地优先 + 常驻 + 采集"场景更合适（体积、权限、内存）。

---

## 13. MVP 范围 (v0.1)

### 13.1 v0.1 必须包含
1. 采集：应用活动 + 事件触发式截图（单屏）
2. 事件聚合 → 工作段
3. SQLite 存储与导出
4. AI 理解：接一个云模型，做分类 + 摘要
5. 个人记忆 / 自定义分类管理
6. 工作时间线 UI
7. 日报生成（固定模板）
8. 隐私控制：暂停 / 排除应用 / 截图分析后删除
9. 模型管理：云 + 自定义接口

### 13.2 不在 v0.1
- 周报/月报模板系统、模板市场
- 可视化报告、网页 AI 对话
- 热力图、词云、专注时段
- 待办提取与完成评估
- Lobster 对外 API（含鉴权/写接口）、Agent 接入
- 模型网页账号接入（放 v0.3）
- 多模态视觉 fallback（放 v0.2）
- 小程序、跨设备同步
- 团队/共享待办协作流（v1.0 前不做）

---

## 14. 里程碑

| 版本 | 目标 | 关键交付 |
|---|---|---|
| v0.1 | 跑通核心闭环（记录→理解→日报） | 采集 + 聚合 + AI 分类 + 时间线 + 日报 + 隐私控制 + 本地 OCR + 内部只读 HTTP 骨架 |
| v0.2 | 报告体系 + 洞察 + 待办 | 模板系统、周/月报、热力图、应用统计、待办提取、多模态 fallback（可选） |
| v0.3 | 闭环完整 + 可视化 + 开放 | 待办完成评估、可视化报告、网页对话、完整 Lobster API、Agent 接入、模型网页账号接入 |
| v1.0 | 多端 + 同步 | 微信小程序、跨设备同步（可选） |
| v2.0 | 团队协作版 | 账号体系、共享待办、协作流（独立产品线） |

---

## 15. 最易被忽略的三个点（实施警示）

1. **个人记忆（memories）**：决定分类与报告质量上限，v0.1 必须做，不能后置
2. **完成评估回环**：不做就只是日报工具，闭环灵魂，v0.3 必须实现
3. **事件聚合策略**：直接决定 token 成本与报告可读性，原始事件不能直送 AI

---

## 16. 设计决策（已定）

### 16.1 截图 OCR 策略：混合策略，默认本地
- **决策**：本地 `RapidOCR` 提取文字 + 文本 LLM 做语义理解（v0.1 默认）
- 仅当文本 LLM 无法判断（纯图表/设计稿/无文字 UI）且用户显式开启多模态时，才 fallback 到云视觉模型（v0.2 可选项）
- **理由**：默认省钱省隐私；纯多模态 token 爆炸，纯 OCR 丢失语义
- 关键实现点：OCR 文本与"应用标题/进程名"一起喂给文本 LLM，覆盖 90% 场景

### 16.2 Lobster API 范围：v0.1 内部只读骨架
- **决策**：v0.1 仅做"内部 HTTP 只读骨架"（守护进程 ↔ UI 通信），监听 127.0.0.1，无 token
- 完整 Lobster（Bearer 鉴权、写接口、Agent 协议、白名单）放 v0.3
- **理由**：通信本就需要 IPC，HTTP 最自然，顺手打地基避免后期重构；但 v0.1 不对外承诺，避免过早暴露写接口

### 16.3 模型网页账号接入：v0.1 不做，后置 v0.3
- **决策**：v0.1 不做网页账号接入；放 v0.3 作为差异化特性
- v0.1 模型来源仅三路：云 API + 自定义接口（OpenAI 兼容）+ 本地 Ollama
- **理由**：网页账号 Cookie 管理、多平台适配、反爬风控复杂且易失效；v0.1 已重，会拖慢 MVP；三路已覆盖 90% 用户

### 16.4 团队/共享待办：v1.0 前不做，团队版作 v2.0
- **决策**：v1.0 锁定个人版；团队协作版作为 v2.0 独立产品线
- 待办 `assignee` 字段保留为**纯文本**（可填他人名字），但不做协作流/通知/共享
- **理由**：团队协作与"本地优先 + 不做 SaaS 多租户"非目标直接冲突；引入账号体系会破坏隐私核心卖点

### 16.5 录屏权限统一抽象：trait + 平台实现 + 降级采集
- **决策**：定义 `CapturePermission` trait + `ScreenCapture` trait，macOS/Windows 各一实现
- **关键原则：权限被拒不阻断整体采集**，降级为"只记应用活动 + 窗口标题，不截图"，保证闭环不中断
- UI 侧统一 `PermissionBadge` 组件，三态：已授权 / 未授权（点击引导）/ 受限（降级采集中）

```rust
enum PermissionState { Granted, Denied, NotDetermined, Restricted }

trait CapturePermission {
    fn check(&self) -> PermissionState;
    fn request(&self) -> Result<PermissionState>;
    fn monitor(&self) -> impl Stream<Item = PermissionState>;
}

trait ScreenCapture {
    fn capture(&self, display: DisplayId) -> Result<Image>;
}
```
- macOS：`CGPreflightScreenCapture` / `CGRequestScreenCaptureAccess`
- Windows：`Graphics.CaptureItem`，首次弹确认框，记住勾选后持久化

---

## 附录 A：术语表

| 术语 | 含义 |
|---|---|
| 工作段 (work_segment) | AI 聚合后的一段连续工作，含起止时间、应用、摘要 |
| 个人记忆 (memory) | 用户的分类、偏好、长期事实，影响 AI 输出 |
| Lobster | 本地开放 HTTP 服务的代号；v0.1 仅内部只读，v0.3 完整对外 |
| 闭环 | 记录→理解→汇报→推进→回写的完整循环 |
| 本地优先 | 数据默认存本地，上云能力默认关闭 |
| 降级采集 | 录屏权限被拒时，仅记应用活动不截图，闭环不中断 |
| Provider 抽象 | 统一 LLM 接口，支持云/自定义/本地/网页四类来源 |

## 附录 B：参考

本项目为学习复刻，从零设计与实现，不含任何第三方专有资源。
