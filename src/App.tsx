import { useEffect, useState } from "react";
import { api } from "./lib";
import Timeline from "./components/Timeline";
import ReportView from "./components/ReportView";
import StatsView from "./components/StatsView";
import SettingsView from "./components/SettingsView";
import TodosView from "./components/TodosView";

type Tab = "timeline" | "report" | "todos" | "stats" | "settings";

const TABS: { id: Tab; label: string }[] = [
  { id: "timeline", label: "时间线" },
  { id: "report", label: "日报" },
  { id: "todos", label: "待办" },
  { id: "stats", label: "统计" },
  { id: "settings", label: "设置" },
];

export default function App() {
  const [tab, setTab] = useState<Tab>("timeline");
  const [paused, setPaused] = useState(false);
  const [loadingPause, setLoadingPause] = useState(true);
  const [dark, setDark] = useState(false);

  useEffect(() => {
    api.getSettings().then((s) => {
      setPaused(s.paused === "true");
      setDark(s.theme === "dark");
      setLoadingPause(false);
    });
  }, []);

  const togglePause = async () => {
    const next = !paused;
    setPaused(next);
    await api.setPaused(next);
  };

  const toggleDark = async () => {
    const next = !dark;
    setDark(next);
    await api.saveSetting("theme", next ? "dark" : "light");
  };

  return (
    <div className={`h-screen flex flex-col bg-neutral-50 ${dark ? "dark" : ""}`}>
      <header className="h-14 bg-white/90 border-b border-black/5 flex items-center px-5 gap-6 shrink-0">
        <div className="flex items-center gap-2">
          <div className="w-7 h-7 rounded-lg bg-primary flex items-center justify-center text-white text-sm font-bold">
            D
          </div>
          <span className="font-semibold text-sm">Daily Trace</span>
          <span className="text-[10px] text-neutral-400 ml-1">v0.1 本地优先</span>
        </div>
        <nav className="flex items-center gap-1">
          {TABS.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={`px-3 py-1.5 text-sm rounded-md transition ${
                tab === t.id
                  ? "bg-primary-light text-primary-dark font-semibold"
                  : "text-neutral-500 hover:text-neutral-800 hover:bg-neutral-100"
              }`}
            >
              {t.label}
            </button>
          ))}
        </nav>
        <div className="flex-1" />
        <button
          onClick={toggleDark}
          className="px-2.5 py-1.5 text-xs rounded-md border border-neutral-200 hover:border-neutral-400 text-neutral-500"
          title="切换深色/浅色"
        >
          {dark ? "浅色" : "深色"}
        </button>
        <button
          onClick={togglePause}
          disabled={loadingPause}
          className={`px-3 py-1.5 text-xs rounded-md font-medium border transition ${
            paused
              ? "bg-amber-50 border-amber-300 text-amber-700"
              : "bg-white border-neutral-300 text-neutral-600 hover:border-neutral-400"
          }`}
        >
          {loadingPause ? "…" : paused ? "● 已暂停记录" : "○ 记录中"}
        </button>
      </header>

      <main className="flex-1 overflow-auto">
        {tab === "timeline" && <Timeline />}
        {tab === "report" && <ReportView />}
        {tab === "todos" && <TodosView />}
        {tab === "stats" && <StatsView />}
        {tab === "settings" && <SettingsView />}
      </main>
    </div>
  );
}
