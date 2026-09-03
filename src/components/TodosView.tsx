import { useEffect, useState } from "react";
import { api, type Todo } from "../lib";

const PERIODS = [
  { id: "day", label: "今日" },
  { id: "week", label: "本周" },
] as const;

export default function TodosView() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [period, setPeriod] = useState<"day" | "week">("day");
  const [loading, setLoading] = useState("");
  const [msg, setMsg] = useState("");

  const load = () => api.listTodos().then(setTodos);
  useEffect(() => {
    load();
  }, []);

  const open = todos.filter((t) => t.status === "open" || t.status === "doing");
  const done = todos.filter((t) => t.status === "done" || t.status === "skipped");

  const onGen = async () => {
    setLoading("gen");
    setMsg("");
    try {
      const t = await api.generateTodos(period);
      setTodos(t);
      setMsg(
        `已从${period === "day" ? "今日" : "本周"}工作记录提取 ${
          t.filter((x) => x.status === "open" || x.status === "doing").length
        } 个待办`
      );
    } catch (e) {
      setMsg(String(e));
    } finally {
      setLoading("");
    }
  };

  const onEval = async () => {
    setLoading("eval");
    setMsg("");
    try {
      const n = await api.evaluateTodos();
      await load();
      setMsg(
        n > 0
          ? `基于最近 24 小时工作记录，评估 ${n} 个待办已推进完成`
          : "暂无待办被评估为已完成"
      );
    } catch (e) {
      setMsg(String(e));
    } finally {
      setLoading("");
    }
  };

  const toggle = async (t: Todo) => {
    const next = t.status === "done" ? "open" : "done";
    await api.updateTodo(t.id, next);
    await load();
  };

  return (
    <div className="max-w-3xl mx-auto p-6">
      <div className="mb-4">
        <h2 className="text-lg font-bold">待办与推进</h2>
        <p className="text-xs text-neutral-500 mt-0.5">
          AI 从工作记录提取待办，并基于后续记录评估完成情况——闭环的灵魂
        </p>
      </div>

      <div className="bg-white border border-neutral-200 rounded-lg p-4 mb-5">
        <div className="flex items-center gap-3 mb-3">
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
            onClick={onEval}
            disabled={!!loading}
            className="px-3 py-1.5 text-xs rounded-md border border-neutral-300 hover:border-neutral-400 disabled:opacity-50"
          >
            {loading === "eval" ? "评估中…" : "评估完成度"}
          </button>
          <button
            onClick={onGen}
            disabled={!!loading}
            className="px-4 py-1.5 text-sm rounded-md bg-primary text-white font-medium hover:bg-primary-dark disabled:opacity-50"
          >
            {loading === "gen" ? "提取中…" : "提取待办"}
          </button>
        </div>
        <p className="text-[11px] text-neutral-400 leading-relaxed">
          「提取待办」从所选周期工作记录中由 AI 提取候选事项；「评估完成度」基于最近
          24 小时工作记录判断开放待办是否已推进完成。
        </p>
        {msg && <p className="text-xs text-primary-dark mt-2">{msg}</p>}
      </div>

      <section className="mb-5">
        <h3 className="text-sm font-semibold text-neutral-700 mb-2">
          待推进（{open.length}）
        </h3>
        {open.length === 0 ? (
          <div className="p-6 text-center text-xs text-neutral-400 bg-white border border-dashed border-neutral-200 rounded-lg">
            暂无待推进事项，点击「提取待办」从工作记录生成
          </div>
        ) : (
          <div className="space-y-2">
            {open.map((t) => (
              <div
                key={t.id}
                className="flex items-center gap-3 p-3 bg-white border border-neutral-200 rounded-lg"
              >
                <button
                  onClick={() => toggle(t)}
                  className="w-5 h-5 rounded-full border-2 border-neutral-300 hover:border-primary shrink-0"
                  title="标记完成"
                />
                <div className="flex-1">
                  <p className="text-sm text-neutral-800">{t.title}</p>
                  <p className="text-[11px] text-neutral-400 mt-0.5">
                    {t.description ?? "基于工作段记录提取"}
                    {t.source_seg_id ? ` · 来源段 #${t.source_seg_id}` : ""}
                  </p>
                </div>
                <span className="text-[10px] px-2 py-0.5 rounded-full bg-amber-50 text-amber-700 border border-amber-200">
                  {t.status}
                </span>
              </div>
            ))}
          </div>
        )}
      </section>

      <section>
        <h3 className="text-sm font-semibold text-neutral-700 mb-2">
          已完成（{done.length}）
        </h3>
        {done.length === 0 ? (
          <p className="text-xs text-neutral-400">暂无已完成事项</p>
        ) : (
          <div className="space-y-2">
            {done.map((t) => (
              <div
                key={t.id}
                className="flex items-center gap-3 p-3 bg-neutral-50 border border-neutral-200 rounded-lg"
              >
                <button
                  onClick={() => toggle(t)}
                  className="w-5 h-5 rounded-full bg-primary flex items-center justify-center shrink-0"
                  title="重新打开"
                >
                  <svg
                    width="12"
                    height="12"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="white"
                    strokeWidth="3"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                </button>
                <p className="text-sm text-neutral-400 line-through flex-1">
                  {t.title}
                </p>
                <span className="text-[10px] text-neutral-400">{t.status}</span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
