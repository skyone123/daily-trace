import { useEffect, useState } from "react";
import {
  api,
  dayBounds,
  fmtDuration,
  fmtTime,
  type TimelineEvent,
  type WorkSegment,
} from "../lib";

export default function Timeline() {
  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [segments, setSegments] = useState<WorkSegment[]>([]);
  const [loading, setLoading] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(true);

  const load = async () => {
    const [from, to] = dayBounds();
    let evs = await api.listTimeline(from, to, 500);
    if (evs.length === 0) {
      await api.seedDemoData();
      await api.aggregateRange(from, to);
      evs = await api.listTimeline(from, to, 500);
    }
    const segs = await api.listSegments(from, to);
    setEvents(evs);
    setSegments(segs);
  };

  useEffect(() => {
    load();
    if (!autoRefresh) return;
    const t = setInterval(load, 5000);
    return () => clearInterval(t);
  }, [autoRefresh]);

  const onAggregate = async () => {
    setLoading(true);
    try {
      const [from, to] = dayBounds();
      await api.aggregateRange(from, to);
      await load();
    } finally {
      setLoading(false);
    }
  };

  const onSeed = async () => {
    await api.seedDemoData();
    await onAggregate();
  };

  const totalMs = events.reduce(
    (sum, e) =>
      sum + ((e.ts_end ?? e.ts_start) - e.ts_start),
    0
  );

  return (
    <div className="max-w-5xl mx-auto p-6">
      <div className="flex items-center justify-between mb-5">
        <div>
          <h2 className="text-lg font-bold">工作时间线</h2>
          <p className="text-xs text-neutral-500 mt-0.5">
            今日采集 {events.length} 个事件，{segments.length} 个工作段，总时长{" "}
            {fmtDuration(totalMs)}
          </p>
        </div>
        <div className="flex gap-2">
          <label className="flex items-center gap-1.5 text-xs text-neutral-500">
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
              className="accent-primary"
            />
            自动刷新
          </label>
          <button
            onClick={onSeed}
            className="px-3 py-1.5 text-xs rounded-md border border-neutral-300 bg-white hover:border-neutral-400"
          >
            灌入演示数据
          </button>
          <button
            onClick={onAggregate}
            disabled={loading}
            className="px-3 py-1.5 text-xs rounded-md bg-primary text-white font-medium hover:bg-primary-dark disabled:opacity-50"
          >
            {loading ? "聚合中…" : "重新聚合"}
          </button>
        </div>
      </div>

      {segments.length > 0 && (
        <section className="mb-6">
          <h3 className="text-sm font-semibold text-neutral-700 mb-2">
            工作段（AI 聚合后）
          </h3>
          <div className="space-y-2">
            {segments.map((s) => (
              <div
                key={s.id}
                className="flex items-start gap-3 p-3 bg-white border border-neutral-200 rounded-lg"
              >
                <div className="text-xs text-neutral-400 w-24 shrink-0 pt-0.5">
                  {fmtTime(s.ts_start)} - {fmtTime(s.ts_end)}
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-semibold px-2 py-0.5 rounded bg-primary-light text-primary-dark">
                      {s.category ?? "未知"}
                    </span>
                    <span className="text-xs text-neutral-400">
                      {fmtDuration(s.ts_end - s.ts_start)}
                    </span>
                  </div>
                  {s.summary && (
                    <p className="text-xs text-neutral-600 mt-1">{s.summary}</p>
                  )}
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      <section>
        <h3 className="text-sm font-semibold text-neutral-700 mb-2">
          原始事件流
        </h3>
        {events.length === 0 ? (
          <div className="p-8 text-center text-sm text-neutral-400 bg-white border border-dashed border-neutral-200 rounded-lg">
            今日暂无记录。可点击「灌入演示数据」生成测试事件，
            或正常使用电脑（应用切换会自动采集）。
          </div>
        ) : (
          <div className="bg-white border border-neutral-200 rounded-lg divide-y divide-neutral-100">
            {events.map((e) => {
              const dur = (e.ts_end ?? e.ts_start) - e.ts_start;
              return (
                <div key={e.id} className="flex items-center gap-3 px-3 py-2">
                  <span className="text-xs text-neutral-400 w-20 shrink-0">
                    {fmtTime(e.ts_start)}
                  </span>
                  <span className="text-xs font-medium text-neutral-700 w-40 truncate">
                    {e.app_name ?? "-"}
                  </span>
                  <span className="text-xs text-neutral-500 flex-1 truncate">
                    {e.window_title ?? ""}
                  </span>
                  <span className="text-xs text-neutral-400">
                    {dur > 0 ? fmtDuration(dur) : "进行中"}
                  </span>
                </div>
              );
            })}
          </div>
        )}
      </section>
    </div>
  );
}
