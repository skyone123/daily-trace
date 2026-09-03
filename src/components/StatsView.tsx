import { useEffect, useState } from "react";
import {
  api,
  dayBounds,
  fmtDuration,
  fmtTime,
  type AppStat,
  type FocusSession,
  type HeatCell,
  type WordFreq,
} from "../lib";

const COLORS = [
  "#16a34a", "#0891b2", "#7c3aed", "#db2777", "#ea580c",
  "#ca8a04", "#0284c7", "#4f46e5", "#16a34a", "#9333ea",
];

const DAYS = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"];
const HOURS = Array.from({ length: 24 }, (_, i) => i);

function weekBounds(): [number, number] {
  const d = new Date();
  const weekday = (d.getDay() + 6) % 7;
  const monday = new Date(d);
  monday.setDate(d.getDate() - weekday);
  monday.setHours(0, 0, 0, 0);
  return [monday.getTime(), Date.now()];
}

function heatColor(level: number): string {
  return ["#f3f4f6", "#bbf7d0", "#86efac", "#4ade80", "#16a34a"][
    Math.min(4, Math.max(0, level))
  ];
}

export default function StatsView() {
  const [stats, setStats] = useState<AppStat[]>([]);
  const [heat, setHeat] = useState<HeatCell[]>([]);
  const [focus, setFocus] = useState<FocusSession[]>([]);
  const [words, setWords] = useState<WordFreq[]>([]);
  const [loading, setLoading] = useState(true);

  const load = async () => {
    const [dFrom, dTo] = dayBounds();
    const [wFrom, wTo] = weekBounds();
    const [s, h, f, wc] = await Promise.all([
      api.statsByApp(dFrom, dTo),
      api.statsHeatmap(wFrom, wTo),
      api.statsFocus(wFrom, wTo),
      api.statsWordcloud(wFrom, wTo),
    ]);
    setStats(s);
    setHeat(h);
    setFocus(f);
    setWords(wc);
    setLoading(false);
  };

  useEffect(() => {
    load();
    const t = setInterval(load, 30000);
    return () => clearInterval(t);
  }, []);

  const heatMap = new Map<string, number>();
  heat.forEach((c) => heatMap.set(`${c.day}-${c.hour}`, c.ms));
  const maxHeat = Math.max(...heat.map((c) => c.ms), 1);

  const totalMs = stats.reduce((a, s) => a + s.ms, 0);
  const maxMs = Math.max(...stats.map((s) => s.ms), 1);
  const maxWord = Math.max(...words.map((w) => w.count), 1);

  if (loading) {
    return <div className="p-6 text-sm text-neutral-400">加载中…</div>;
  }

  return (
    <div className="max-w-4xl mx-auto p-6 space-y-5">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold">工作洞察</h2>
        <span className="text-xs text-neutral-500">
          今日 {fmtDuration(totalMs)} · 本周热力图与专注时段
        </span>
      </div>

      <section className="bg-white border border-neutral-200 rounded-lg p-4">
        <h3 className="text-sm font-semibold text-neutral-700 mb-3">
          工作热力图（本周 · 按小时）
        </h3>
        <div className="flex gap-2 overflow-x-auto">
          <div className="flex flex-col gap-1 mr-1 text-[10px] text-neutral-400 pt-0">
            {DAYS.map((d) => (
              <div key={d} className="h-4 flex items-center leading-none">
                {d}
              </div>
            ))}
          </div>
          <div>
            <div className="flex flex-col gap-1">
              {DAYS.map((d, di) => (
                <div key={di} className="flex gap-1">
                  {HOURS.map((h) => {
                    const ms = heatMap.get(`${di}-${h}`) || 0;
                    const level = ms === 0 ? 0 : Math.ceil((ms / maxHeat) * 4);
                    return (
                      <div
                        key={h}
                        title={`${d} ${String(h).padStart(2, "0")}:00 · ${
                          ms ? fmtDuration(ms) : "无记录"
                        }`}
                        className="w-3.5 h-4 rounded-sm"
                        style={{ backgroundColor: heatColor(level) }}
                      />
                    );
                  })}
                </div>
              ))}
            </div>
            <div className="flex items-center gap-1 mt-2 ml-1 text-[10px] text-neutral-400">
              <span>少</span>
              {[0, 1, 2, 3, 4].map((l) => (
                <div
                  key={l}
                  className="w-3 h-3 rounded-sm"
                  style={{ backgroundColor: heatColor(l) }}
                />
              ))}
              <span>多</span>
            </div>
          </div>
        </div>
      </section>

      <section className="bg-white border border-neutral-200 rounded-lg p-4">
        <h3 className="text-sm font-semibold text-neutral-700 mb-3">
          应用统计（今日）
        </h3>
        {stats.length === 0 ? (
          <p className="text-xs text-neutral-400">今日暂无统计数据</p>
        ) : (
          <div className="space-y-2">
            {stats.map((s, i) => {
              const pct = Math.max(4, (s.ms / maxMs) * 100);
              return (
                <div key={s.app} className="flex items-center gap-3">
                  <div className="w-32 text-xs text-neutral-600 truncate shrink-0">
                    {s.app}
                  </div>
                  <div className="flex-1 bg-neutral-100 rounded h-6 overflow-hidden">
                    <div
                      className="h-full rounded flex items-center justify-end pr-2"
                      style={{
                        width: `${pct}%`,
                        background: COLORS[i % COLORS.length],
                        minWidth: "60px",
                      }}
                    >
                      <span className="text-[10px] text-white font-medium">
                        {fmtDuration(s.ms)}
                      </span>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </section>

      <section className="bg-white border border-neutral-200 rounded-lg p-4">
        <h3 className="text-sm font-semibold text-neutral-700 mb-3">
          专注时段排行榜（本周）
        </h3>
        {focus.length === 0 ? (
          <p className="text-xs text-neutral-400">本周暂无专注时段</p>
        ) : (
          <ol className="space-y-2">
            {focus.slice(0, 5).map((f, i) => (
              <li
                key={i}
                className="flex items-center gap-3 p-3 bg-neutral-50 border border-neutral-200 rounded-lg"
              >
                <span className="text-primary font-bold text-sm w-6">
                  #{i + 1}
                </span>
                <div className="flex-1">
                  <p className="text-sm text-neutral-800">
                    {fmtTime(f.ts_start)} - {fmtTime(f.ts_end)} ·{" "}
                    {fmtDuration(f.ms)}
                  </p>
                  <p className="text-[11px] text-neutral-400 mt-0.5">
                    {f.seg_count} 个工作段 · {f.apps.join(" / ")}
                  </p>
                </div>
              </li>
            ))}
          </ol>
        )}
      </section>

      <section className="bg-white border border-neutral-200 rounded-lg p-4">
        <h3 className="text-sm font-semibold text-neutral-700 mb-3">
          工作词云（本周高频标题）
        </h3>
        {words.length === 0 ? (
          <p className="text-xs text-neutral-400">暂无词频数据</p>
        ) : (
          <div className="flex flex-wrap gap-2.5 items-center justify-center py-4">
            {words.map((w) => {
              const size = 12 + Math.round((w.count / maxWord) * 18);
              const opacity = 0.55 + (w.count / maxWord) * 0.45;
              return (
                <span
                  key={w.word}
                  className="text-primary-dark font-medium"
                  style={{ fontSize: `${size}px`, opacity }}
                >
                  {w.word}
                </span>
              );
            })}
          </div>
        )}
      </section>
    </div>
  );
}
