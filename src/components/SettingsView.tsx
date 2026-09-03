import { useEffect, useState } from "react";
import { api, type Settings } from "../lib";

const isTauri =
  typeof window !== "undefined" &&
  ((window as any).__TAURI_INTERNALS__ || (window as any).__TAURI__);

export default function SettingsView() {
  const [s, setS] = useState<Settings>({});
  const [cats, setCats] = useState<string[]>([]);
  const [saved, setSaved] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testMsg, setTestMsg] = useState("");
  const [exporting, setExporting] = useState(false);
  const [exportMsg, setExportMsg] = useState("");
  const [updating, setUpdating] = useState(false);
  const [updateMsg, setUpdateMsg] = useState("");

  useEffect(() => {
    api.getSettings().then(setS);
    api.listCategories().then(setCats);
  }, []);

  const update = (k: string, v: string) =>
    setS((prev) => ({ ...prev, [k]: v }));

  const save = async () => {
    await Promise.all(
      Object.entries(s).map(([k, v]) => api.saveSetting(k, String(v)))
    );
    setSaved(true);
    setTimeout(() => setSaved(false), 1800);
  };

  const testModel = async () => {
    setTesting(true);
    setTestMsg("");
    await save();
    try {
      const r = await api.generateReport("day", "这是一次连通性测试，请简短确认。");
      setTestMsg(`✓ 成功，模型 ${r.model ?? "-"}`);
    } catch (e) {
      setTestMsg(`✗ ${String(e)}`);
    } finally {
      setTesting(false);
    }
  };

  const onExport = async () => {
    setExporting(true);
    setExportMsg("");
    try {
      const json = await api.exportData();
      if (isTauri) {
        const { save } = await import("@tauri-apps/plugin-dialog");
        const { writeTextFile } = await import("@tauri-apps/plugin-fs");
        const path = await save({
          defaultPath: `daily-trace-backup-${new Date().toISOString().slice(0, 10)}.json`,
          filters: [{ name: "JSON", extensions: ["json"] }],
        });
        if (path) {
          await writeTextFile(path, json);
          setExportMsg("已导出到: " + path);
        }
      } else {
        const blob = new Blob([json], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `daily-trace-backup-${new Date().toISOString().slice(0, 10)}.json`;
        a.click();
        URL.revokeObjectURL(url);
        setExportMsg("已导出");
      }
    } catch (e) {
      setExportMsg("导出失败: " + String(e));
    } finally {
      setExporting(false);
    }
  };

  const checkUpdate = async () => {
    setUpdating(true);
    setUpdateMsg("");
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const { relaunch } = await import("@tauri-apps/plugin-process");
      const update = await check();
      if (update) {
        setUpdateMsg(`发现新版本 ${update.version}，正在下载安装...`);
        await update.downloadAndInstall();
        setUpdateMsg("更新已安装，即将重启...");
        await relaunch();
      } else {
        setUpdateMsg("已是最新版本");
      }
    } catch (e) {
      setUpdateMsg("检查失败: " + String(e));
    } finally {
      setUpdating(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto p-6 space-y-5">
      <h2 className="text-lg font-bold">设置</h2>

      <Card title="模型配置" desc="决定日报生成质量。Mock 模式离线生成，配置 API Key 后启用 AI 智能归类。">
        <Field label="模型来源">
          <select
            value={s.model_kind ?? "mock"}
            onChange={(e) => update("model_kind", e.target.value)}
            className="input"
          >
            <option value="mock">Mock 离线模式（默认）</option>
            <option value="openai">OpenAI 兼容云模型</option>
          </select>
        </Field>
        {(s.model_kind ?? "mock") !== "mock" && (
          <>
            <Field label="Base URL">
              <input
                value={s.model_base_url ?? ""}
                onChange={(e) => update("model_base_url", e.target.value)}
                placeholder="https://api.openai.com/v1"
                className="input"
              />
            </Field>
            <Field label="API Key">
              <input
                type="password"
                value={s.model_api_key ?? ""}
                onChange={(e) => update("model_api_key", e.target.value)}
                placeholder="sk-..."
                className="input"
              />
            </Field>
            <Field label="模型名称">
              <input
                value={s.model_name ?? ""}
                onChange={(e) => update("model_name", e.target.value)}
                placeholder="gpt-4o-mini"
                className="input"
              />
            </Field>
            <button
              onClick={testModel}
              disabled={testing}
              className="mt-1 px-3 py-1.5 text-xs rounded-md border border-neutral-300 hover:border-neutral-400 disabled:opacity-50"
            >
              {testing ? "测试中…" : "保存并测试连通性"}
            </button>
            {testMsg && (
              <p className="text-xs mt-2 text-neutral-600">{testMsg}</p>
            )}
          </>
        )}
      </Card>

      <Card title="隐私与采集" desc="本地优先原则：所有控制默认保护隐私，截图分析后删除。">
        <Field label="排除应用（逗号分隔，匹配进程名或窗口标题）">
          <input
            value={s.excluded_apps ?? ""}
            onChange={(e) => update("excluded_apps", e.target.value)}
            placeholder="如 1Password, 銀行, 微信"
            className="input"
          />
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="采集间隔 (毫秒)">
            <input
              value={s.capture_interval_ms ?? ""}
              onChange={(e) => update("capture_interval_ms", e.target.value)}
              className="input"
            />
          </Field>
          <Field label="空闲阈值 (毫秒)">
            <input
              value={s.idle_threshold_ms ?? ""}
              onChange={(e) => update("idle_threshold_ms", e.target.value)}
              className="input"
            />
          </Field>
        </div>
        <label className="flex items-center gap-2 text-sm text-neutral-700 mt-1">
          <input
            type="checkbox"
            checked={s.delete_screenshot_after === "true"}
            onChange={(e) =>
              update("delete_screenshot_after", String(e.target.checked))
            }
            className="accent-primary"
          />
          截图分析后删除（v0.1 未启用截图采集，预留）
        </label>
      </Card>

      <Card title="工作分类（个人记忆）" desc="影响 AI 归类与报告结构，v0.1 内置默认分类，后续支持自定义。">
        <div className="flex flex-wrap gap-2">
          {cats.map((c) => (
            <span
              key={c}
              className="px-2.5 py-1 text-xs rounded-full bg-primary-light text-primary-dark"
            >
              {c}
            </span>
          ))}
        </div>
      </Card>

      <Card title="数据管理" desc="本地优先：一键导出全部数据为 JSON 备份文件。">
        <button
          onClick={onExport}
          disabled={exporting}
          className="px-3 py-1.5 text-xs rounded-md border border-neutral-300 hover:border-neutral-400 disabled:opacity-50"
        >
          {exporting ? "导出中…" : "导出数据 (JSON)"}
        </button>
        {exportMsg && (
          <p className="text-xs mt-2 text-neutral-600 break-all">{exportMsg}</p>
        )}
      </Card>

      <Card title="关于与更新" desc="从 GitHub Release 自动检查并安装更新。">
        <button
          onClick={checkUpdate}
          disabled={updating}
          className="px-3 py-1.5 text-xs rounded-md border border-neutral-300 hover:border-neutral-400 disabled:opacity-50"
        >
          {updating ? "检查中…" : "检查更新"}
        </button>
        {updateMsg && (
          <p className="text-xs mt-2 text-neutral-600 break-all">{updateMsg}</p>
        )}
      </Card>

      <div className="flex items-center gap-3 pt-2">
        <button
          onClick={save}
          className="px-4 py-2 text-sm rounded-md bg-primary text-white font-medium hover:bg-primary-dark"
        >
          保存设置
        </button>
        {saved && (
          <span className="text-xs text-primary-dark">已保存 ✓</span>
        )}
      </div>

      <style>{`
        .input {
          width: 100%;
          padding: 6px 10px;
          border: 1px solid #e5e5e5;
          border-radius: 6px;
          font-size: 13px;
          background: white;
        }
        .input:focus { outline: none; border-color: #16a34a; }
      `}</style>
    </div>
  );
}

function Card({
  title,
  desc,
  children,
}: {
  title: string;
  desc?: string;
  children: React.ReactNode;
}) {
  return (
    <section className="bg-white border border-neutral-200 rounded-lg p-4">
      <h3 className="text-sm font-semibold text-neutral-800">{title}</h3>
      {desc && <p className="text-[11px] text-neutral-400 mt-0.5 mb-3">{desc}</p>}
      <div className={desc ? "" : "mt-3"}>{children}</div>
    </section>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="mb-3">
      <label className="block text-xs text-neutral-500 mb-1">{label}</label>
      {children}
    </div>
  );
}
