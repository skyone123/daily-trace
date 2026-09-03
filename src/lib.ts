import { invoke } from "@tauri-apps/api/core";

export interface TimelineEvent {
  id: number;
  ts_start: number;
  ts_end: number | null;
  kind: string;
  app_name: string | null;
  window_title: string | null;
  content_ref: string | null;
  raw_meta: string | null;
}

export interface WorkSegment {
  id: number;
  ts_start: number;
  ts_end: number;
  event_ids: string;
  summary: string | null;
  category: string | null;
  tags: string | null;
  focus_score: number | null;
  ai_model: string | null;
}

export interface DailyReport {
  id: number;
  period: string;
  period_start: number;
  period_end: number;
  content: string;
  model: string | null;
  created_at: number;
}

export interface AppStat {
  app: string;
  ms: number;
}

export interface Todo {
  id: number;
  title: string;
  description: string | null;
  assignee: string | null;
  due_at: number | null;
  parent_id: number | null;
  status: string;
  source_seg_id: number | null;
  attachments: string | null;
  created_at: number;
  updated_at: number;
}

export interface HeatCell {
  day: number;
  hour: number;
  ms: number;
}

export interface FocusSession {
  ts_start: number;
  ts_end: number;
  ms: number;
  seg_count: number;
  apps: string[];
}

export interface WordFreq {
  word: string;
  count: number;
}

export type Settings = Record<string, string>;

const isTauri =
  typeof window !== "undefined" &&
  ((window as any).__TAURI_INTERNALS__ || (window as any).__TAURI__);

const _today = new Date();
const at = (h: number, m: number) =>
  new Date(_today.getFullYear(), _today.getMonth(), _today.getDate(), h, m).getTime();
const MIN = 60_000;

const DEMO_EVENTS: TimelineEvent[] = [
  { id: 1, ts_start: at(9, 5), ts_end: at(9, 42), kind: "app", app_name: "VS Code", window_title: "main.rs - daily-trace", content_ref: null, raw_meta: null },
  { id: 2, ts_start: at(9, 42), ts_end: at(10, 15), kind: "app", app_name: "Chrome", window_title: "Daily Trace - 官网", content_ref: null, raw_meta: null },
  { id: 3, ts_start: at(10, 15), ts_end: at(10, 50), kind: "app", app_name: "微信", window_title: "工作群", content_ref: null, raw_meta: null },
  { id: 4, ts_start: at(10, 50), ts_end: at(11, 30), kind: "app", app_name: "Notion", window_title: "需求文档 v2", content_ref: null, raw_meta: null },
  { id: 5, ts_start: at(11, 30), ts_end: at(12, 10), kind: "app", app_name: "Terminal", window_title: "cargo build", content_ref: null, raw_meta: null },
  { id: 6, ts_start: at(13, 30), ts_end: at(14, 15), kind: "app", app_name: "VS Code", window_title: "lib.rs - daily-trace", content_ref: null, raw_meta: null },
  { id: 7, ts_start: at(14, 15), ts_end: at(14, 55), kind: "app", app_name: "Figma", window_title: "首页设计稿", content_ref: null, raw_meta: null },
  { id: 8, ts_start: at(14, 55), ts_end: at(15, 40), kind: "app", app_name: "Chrome", window_title: "Tauri 文档", content_ref: null, raw_meta: null },
];

const classifyApp = (app: string): string => {
  if (app === "VS Code" || app === "Terminal") return "研发编码";
  if (app === "Chrome") return "学习与调研";
  if (app === "微信") return "沟通协作";
  if (app === "Notion") return "文档与汇报";
  if (app === "Figma") return "产品与设计";
  return "运维与杂务";
};

const DEMO_SEGMENTS: WorkSegment[] = DEMO_EVENTS.map((e, i) => ({
  id: i + 1,
  ts_start: e.ts_start,
  ts_end: e.ts_end ?? e.ts_start,
  event_ids: JSON.stringify([e.id]),
  summary: `在 ${e.app_name} 工作 ${Math.round(((e.ts_end ?? e.ts_start) - e.ts_start) / MIN)} 分钟，涉及「${e.window_title}」`,
  category: classifyApp(e.app_name ?? ""),
  tags: null,
  focus_score: null,
  ai_model: null,
}));

const DEMO_REPORT_CONTENT = `## 今日工作日报

### 研发编码（125分钟）
- 涉及「main.rs - daily-trace」
- 涉及「lib.rs - daily-trace」
- 涉及「cargo build」

### 学习与调研（78分钟）
- 涉及「Daily Trace - 官网」
- 涉及「Tauri 文档」

### 沟通协作（35分钟）
- 涉及「工作群」

### 文档与汇报（40分钟）
- 涉及「需求文档 v2」

### 产品与设计（40分钟）
- 涉及「首页设计稿」

本日记录工作段 8 条，有效工作时长约 5.3 小时。已按工作主题智能分类汇总。

> 当前为离线 Mock 模式生成的日报。在设置中配置云模型 API Key 后，将获得 AI 智能归类与润色。`;

const DEMO_SETTINGS: Settings = {
  paused: "false", capture_interval_ms: "1500", idle_threshold_ms: "90000",
  delete_screenshot_after: "true", capture_screenshot: "false", excluded_apps: "1Password, 銶行",
  display_id: "0", model_kind: "mock", model_base_url: "https://api.openai.com/v1",
  model_api_key: "", model_name: "gpt-4o-mini",
};

const DEMO_TODOS: Todo[] = [
  { id: 1, title: "完成 main.rs - daily-trace 的待实现逻辑", description: "基于工作段记录提取", assignee: null, due_at: null, parent_id: null, status: "done", source_seg_id: 1, attachments: null, created_at: Date.now() - 3600_000, updated_at: Date.now() - 600_000 },
  { id: 2, title: "整理今日调研资料并归档要点", description: "基于工作段记录提取", assignee: null, due_at: null, parent_id: null, status: "done", source_seg_id: 2, attachments: null, created_at: Date.now() - 3600_000, updated_at: Date.now() - 600_000 },
  { id: 3, title: "跟进工作群中的待回复事项", description: "基于工作段记录提取", assignee: null, due_at: null, parent_id: null, status: "open", source_seg_id: 3, attachments: null, created_at: Date.now() - 3600_000, updated_at: Date.now() - 3600_000 },
  { id: 4, title: "完善需求文档 v2", description: "基于工作段记录提取", assignee: null, due_at: null, parent_id: null, status: "open", source_seg_id: 4, attachments: null, created_at: Date.now() - 3600_000, updated_at: Date.now() - 3600_000 },
  { id: 5, title: "评审首页设计稿并给出反馈", description: "基于工作段记录提取", assignee: null, due_at: null, parent_id: null, status: "open", source_seg_id: 7, attachments: null, created_at: Date.now() - 3600_000, updated_at: Date.now() - 3600_000 },
];

const hm = (day: number, hour: number, mins: number): HeatCell => ({
  day, hour, ms: mins * MIN,
});
const DEMO_HEATMAP: HeatCell[] = [
  hm(1, 9, 40), hm(1, 10, 55), hm(1, 11, 35), hm(1, 14, 45), hm(1, 15, 40), hm(1, 16, 30),
  hm(2, 9, 50), hm(2, 10, 40), hm(2, 11, 30), hm(2, 14, 55), hm(2, 15, 35), hm(2, 16, 40),
  hm(3, 9, 35), hm(3, 10, 45), hm(3, 11, 50), hm(3, 14, 30), hm(3, 15, 55), hm(3, 16, 35),
  hm(4, 9, 45), hm(4, 10, 35), hm(4, 11, 40), hm(4, 14, 50), hm(4, 15, 40), hm(4, 16, 45),
  hm(5, 9, 40), hm(5, 10, 50), hm(5, 11, 35), hm(5, 14, 45), hm(5, 15, 40), hm(5, 16, 30),
  hm(6, 10, 20), hm(6, 14, 15),
];

const DEMO_FOCUS: FocusSession[] = [
  { ts_start: at(9, 5), ts_end: at(12, 10), ms: 185 * MIN, seg_count: 4, apps: ["VS Code", "Chrome", "微信", "Notion"] },
  { ts_start: at(13, 30), ts_end: at(15, 40), ms: 130 * MIN, seg_count: 3, apps: ["VS Code", "Figma", "Chrome"] },
  { ts_start: at(14, 55), ts_end: at(15, 40), ms: 45 * MIN, seg_count: 1, apps: ["Chrome"] },
];

const DEMO_WORDS: WordFreq[] = [
  { word: "daily-trace", count: 6 },
  { word: "main.rs", count: 5 },
  { word: "官网", count: 4 },
  { word: "Tauri", count: 4 },
  { word: "工作群", count: 3 },
  { word: "需求文档", count: 3 },
  { word: "首页设计稿", count: 3 },
  { word: "cargo", count: 2 },
  { word: "build", count: 2 },
  { word: "lib.rs", count: 2 },
  { word: "文档", count: 2 },
  { word: "设计稿", count: 2 },
];

async function inv<T>(cmd: string, args?: any, mock?: T): Promise<T> {
  if (!isTauri) return mock as T;
  try {
    return await invoke<T>(cmd, args);
  } catch {
    return mock as T;
  }
}

export const api = {
  nowMs: () => inv<number>("now_ms", {}, Date.now()),
  listTimeline: (from: number, to: number, limit?: number) =>
    inv<TimelineEvent[]>("list_timeline", { from, to, limit }, DEMO_EVENTS),
  listSegments: (from: number, to: number) =>
    inv<WorkSegment[]>("list_segments", { from, to }, DEMO_SEGMENTS),
  aggregateRange: (from: number, to: number) =>
    inv<number>("aggregate_range", { from, to }, DEMO_SEGMENTS.length),
  generateReport: (period: string, extra: string) =>
    inv<DailyReport>("generate_report", { period, extra }, {
      id: 1, period, period_start: at(0, 0), period_end: Date.now(),
      content: DEMO_REPORT_CONTENT, model: "mock-local", created_at: Date.now(),
    } as DailyReport),
  listReports: (limit?: number) =>
    inv<DailyReport[]>("list_reports", { limit }, [
      { id: 1, period: "day", period_start: at(0, 0), period_end: Date.now(),
        content: DEMO_REPORT_CONTENT, model: "mock-local", created_at: Date.now() },
    ]),
  getSettings: () => inv<Settings>("get_settings", {}, DEMO_SETTINGS),
  saveSetting: (key: string, value: string) =>
    inv<void>("save_setting", { key, value }, undefined as unknown as void),
  setPaused: (paused: boolean) => inv<void>("set_paused", { paused }, undefined as unknown as void),
  listCategories: () => inv<string[]>("list_categories", {},
    ["研发编码", "产品与设计", "沟通协作", "会议", "文档与汇报", "学习与调研", "运维与杂务", "休息"]),
  statsByApp: (from: number, to: number) =>
    inv<AppStat[]>("stats_by_app", { from, to }, [
      { app: "VS Code", ms: 82 * MIN }, { app: "Chrome", ms: 78 * MIN },
      { app: "微信", ms: 35 * MIN }, { app: "Notion", ms: 40 * MIN },
      { app: "Terminal", ms: 40 * MIN }, { app: "Figma", ms: 40 * MIN },
    ]),
  seedDemoData: () => inv<number>("seed_demo_data", {}, 8),
  listTodos: (status?: string) =>
    inv<Todo[]>("list_todos", { status }, DEMO_TODOS.filter((t) => !status || t.status === status)),
  generateTodos: (period: string) =>
    inv<Todo[]>("generate_todos", { period }, DEMO_TODOS),
  updateTodo: (id: number, status: string) =>
    inv<void>("update_todo", { id, status }, undefined as unknown as void),
  evaluateTodos: () =>
    inv<number>("evaluate_todos", {}, DEMO_TODOS.filter((t) => t.status === "open").length),
  statsHeatmap: (from: number, to: number) =>
    inv<HeatCell[]>("stats_heatmap", { from, to }, DEMO_HEATMAP),
  statsFocus: (from: number, to: number) =>
    inv<FocusSession[]>("stats_focus", { from, to }, DEMO_FOCUS),
  statsWordcloud: (from: number, to: number) =>
    inv<WordFreq[]>("stats_wordcloud", { from, to }, DEMO_WORDS),
  deleteReport: (id: number) =>
    inv<void>("delete_report", { id }, undefined as unknown as void),
  clearReports: (keep?: number) =>
    inv<number>("clear_reports", { keep }, 0),
  classifySegments: (from: number, to: number) =>
    inv<number>("classify_segments", { from, to }, 8),
  exportData: () =>
    inv<string>(
      "export_data",
      {},
      JSON.stringify({ version: 1, demo: true, exported_at: Date.now() }, null, 2)
    ),
};

export function fmtTime(ts: number): string {
  const d = new Date(ts);
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  return `${hh}:${mm}`;
}

export function fmtDuration(ms: number): string {
  const mins = Math.max(1, Math.round(ms / 60000));
  if (mins < 60) return `${mins} 分钟`;
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  return m ? `${h} 小时 ${m} 分` : `${h} 小时`;
}

export function fmtDate(ts: number): string {
  const d = new Date(ts);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(
    d.getDate()
  ).padStart(2, "0")}`;
}

export function dayBounds(): [number, number] {
  const d = new Date();
  const start = new Date(d.getFullYear(), d.getMonth(), d.getDate(), 0, 0, 0).getTime();
  return [start, Date.now()];
}
