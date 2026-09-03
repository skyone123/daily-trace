import { useEffect, useState, type MouseEvent } from "react";
import { api, fmtDate, type DailyReport } from "../lib";
import Markdown from "./Markdown";

const PERIODS = [
  { id: "day", label: "日报" },
  { id: "week", label: "周报" },
  { id: "month", label: "月报" },
] as const;

export default function ReportView() {
  const [period, setPeriod] = useState<"day" | "week" | "month">("day");
  const [extra, setExtra] = useState("");
  const [report, setReport] = useState<DailyReport | null>(null);
  const [history, setHistory] = useState<DailyReport[]>([]);
  const [loading, setLoading] = useState(false);
  const [err, setErr] = useState("");
  const [copied, setCopied] = useState(false);

  const refreshHistory = () => api.listReports(20).then(setHistory);

  useEffect(() => {
    refreshHistory();
  }, []);

  const onGen = async () => {
    setLoading(true);
    setErr("");
    try {
      const r = await api.generateReport(period, extra);
      setReport(r);
      refreshHistory();
    } catch (e) {
      setErr(String(e));
    } finally {
      setLoading(false);
    }
  };

  const copy = async () => {
    if (!report) return;
    await navigator.clipboard.writeText(report.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 1800);
  };

  const onDelete = async (id: number, e: MouseEvent) => {
    e.stopPropagation();
    if (!window.confirm("删除这条报告？")) return;
    await api.deleteReport(id);
    if (report?.id === id) setReport(null);
    refreshHistory();
  };

  const onClear = async () => {
    if (!window.confirm("清空全部历史报告？此操作不可撤销。")) return;
    await api.clearReports();
    setReport(null);
    refreshHistory();
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <h2 className="text-lg font-bold mb-4">生成工作报告</h2>

      <div className="bg-white border border-neutral-200 rounded-lg p-4 mb-5">
        <div className="flex items-center gap-4 mb-3">
          <div className="flex gap-1 bg-neutral-100 p-1 rounded-lg">
            {PERIODS.map((p) => (
              <button
                key={p.id}
                onClick={() => setPeriod(p.id)}
                className={`px-3 py-1 text-xs rounded-md transition ${
                  period === p.id
                    ? "bg-white shadow-sm font-semibold text-primary-dark"
                    : "text-neutral-500"
                }`}
              >
                {p.label}
              </button>
            ))}
          </div>
          <div className="flex-1" />
          <button
            onClick={onGen}
            disabled={loading}
            className="px-4 py-1.5 text-sm rounded-md bg-primary text-white font-medium hover:bg-primary-dark disabled:opacity-50"
          >
            {loading ? "生成中…" : "生成报告"}
          </button>
        </div>
        <textarea
          value={extra}
          onChange={(e) => setExtra(e.target.value)}
          placeholder="附加要求（可选）：例如侧重产出、控制字数、强调某模块…"
          className="w-full text-sm p-2 border border-neutral-200 rounded-md resize-none focus:outline-none focus:border-primary"
          rows={2}
        />
        {err && (
          <p className="text-xs text-red-500 mt-2">⚠ {err}</p>
        )}
      </div>

      {report && (
        <div className="bg-white border border-neutral-200 rounded-lg p-5 mb-5">
          <div className="flex items-center justify-between mb-3 pb-3 border-b border-neutral-100">
            <div>
              <h3 className="font-semibold">
                {PERIODS.find((p) => p.id === report.period)?.label} ·{" "}
                {fmtDate(report.period_start)}
              </h3>
              <p className="text-[11px] text-neutral-400 mt-0.5">
                模型：{report.model ?? "-"} · {fmtDate(report.created_at)} 生成
              </p>
            </div>
            <button
              onClick={copy}
              className="px-3 py-1 text-xs rounded-md border border-neutral-300 hover:border-neutral-400"
            >
              {copied ? "已复制 ✓" : "复制全文"}
            </button>
          </div>
          <Markdown content={report.content} />
        </div>
      )}

      <section>
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-sm font-semibold text-neutral-700">历史报告</h3>
          {history.length > 0 && (
            <button
              onClick={onClear}
              className="text-xs text-neutral-400 hover:text-red-500 transition"
            >
              清空历史
            </button>
          )}
        </div>
        {history.length === 0 ? (
          <p className="text-xs text-neutral-400">暂无历史报告</p>
        ) : (
          <div className="bg-white border border-neutral-200 rounded-lg divide-y divide-neutral-100">
            {history.map((r) => (
              <div
                key={r.id}
                onClick={() => setReport(r)}
                className={`w-full flex items-center gap-3 px-3 py-2 text-left hover:bg-neutral-50 cursor-pointer ${
                  report?.id === r.id ? "bg-primary-light/40" : ""
                }`}
              >
                <span className="text-xs font-medium w-12">
                  {PERIODS.find((p) => p.id === r.period)?.label}
                </span>
                <span className="text-xs text-neutral-500">
                  {fmtDate(r.period_start)}
                </span>
                <span className="text-[11px] text-neutral-400 flex-1 truncate">
                  {r.content.split("\n")[0]}
                </span>
                <span className="text-[10px] text-neutral-400">
                  {r.model ?? "-"}
                </span>
                <button
                  onClick={(e) => onDelete(r.id, e)}
                  className="text-neutral-300 hover:text-red-500 shrink-0 transition"
                  title="删除"
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
                </button>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
