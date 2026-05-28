import { useEffect, useState } from "react";
import { useT } from "@/lib/i18n";
import {
  getSettings,
  testProvider,
  updateSettings,
} from "@/lib/tauri";
import type { ProviderKind, Settings, TestProviderResult } from "@/lib/types";

export default function SettingsPage() {
  const { t, lang, setLang } = useT();
  const [settings, setSettings] = useState<Settings | null>(null);
  const [provider, setProvider] = useState<ProviderKind>("gemini");
  const [model, setModel] = useState("");
  const [geminiKey, setGeminiKey] = useState("");
  const [tavilyKey, setTavilyKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<TestProviderResult | null>(null);

  useEffect(() => {
    getSettings()
      .then((s) => {
        setSettings(s);
        setProvider(s.provider);
        setModel(s.model);
      })
      .catch(console.error);
  }, []);

  async function save() {
    setSaving(true);
    try {
      const next = await updateSettings({
        provider,
        model,
        language: lang,
        gemini_api_key: geminiKey ? geminiKey : undefined,
        tavily_api_key: tavilyKey ? tavilyKey : undefined,
      });
      setSettings(next);
      setGeminiKey("");
      setTavilyKey("");
    } catch (err) {
      console.error(err);
    } finally {
      setSaving(false);
    }
  }

  async function runTest() {
    setTesting(true);
    setTestResult(null);
    try {
      const r = await testProvider();
      setTestResult(r);
    } catch (err) {
      setTestResult({ ok: false, message: String(err) });
    } finally {
      setTesting(false);
    }
  }

  return (
    <div className="max-w-2xl mx-auto px-6 py-8 space-y-6">
      <h1 className="text-2xl font-semibold">{t("settings.title")}</h1>

      <section className="space-y-3">
        <label className="block text-sm font-medium">
          {t("settings.provider")}
        </label>
        <select
          value={provider}
          onChange={(e) => setProvider(e.target.value as ProviderKind)}
          className="w-full px-3 py-2 rounded-md border border-zinc-300 dark:border-zinc-700 bg-white dark:bg-zinc-900"
        >
          <option value="gemini">Gemini</option>
          <option value="openai" disabled>
            OpenAI (TODO)
          </option>
          <option value="ollama" disabled>
            Ollama (TODO)
          </option>
        </select>
      </section>

      <section className="space-y-3">
        <label className="block text-sm font-medium">
          {t("settings.model")}
        </label>
        <input
          type="text"
          value={model}
          onChange={(e) => setModel(e.target.value)}
          placeholder="gemini-2.0-flash"
          className="w-full px-3 py-2 rounded-md border border-zinc-300 dark:border-zinc-700 bg-white dark:bg-zinc-900"
        />
      </section>

      <section className="space-y-3">
        <label className="block text-sm font-medium">
          {t("settings.gemini_key")}{" "}
          <span className="ml-2 text-xs text-zinc-500">
            ({settings?.has_gemini_key
              ? t("settings.key_set")
              : t("settings.key_unset")})
          </span>
        </label>
        <input
          type="password"
          value={geminiKey}
          onChange={(e) => setGeminiKey(e.target.value)}
          placeholder="AIza..."
          className="w-full px-3 py-2 rounded-md border border-zinc-300 dark:border-zinc-700 bg-white dark:bg-zinc-900"
        />
      </section>

      <section className="space-y-3">
        <label className="block text-sm font-medium">
          {t("settings.tavily_key")}{" "}
          <span className="ml-2 text-xs text-zinc-500">
            ({settings?.has_tavily_key
              ? t("settings.key_set")
              : t("settings.key_unset")})
          </span>
        </label>
        <input
          type="password"
          value={tavilyKey}
          onChange={(e) => setTavilyKey(e.target.value)}
          placeholder="tvly-..."
          className="w-full px-3 py-2 rounded-md border border-zinc-300 dark:border-zinc-700 bg-white dark:bg-zinc-900"
        />
      </section>

      <section className="space-y-3">
        <label className="block text-sm font-medium">
          {t("settings.language")}
        </label>
        <select
          value={lang}
          onChange={(e) => setLang(e.target.value as "ko" | "en")}
          className="w-full px-3 py-2 rounded-md border border-zinc-300 dark:border-zinc-700 bg-white dark:bg-zinc-900"
        >
          <option value="ko">한국어</option>
          <option value="en">English</option>
        </select>
      </section>

      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={save}
          disabled={saving}
          className="px-4 py-2 rounded-md bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 disabled:opacity-50"
        >
          {saving ? t("common.loading") : t("settings.save")}
        </button>
        <button
          type="button"
          onClick={runTest}
          disabled={testing}
          className="px-4 py-2 rounded-md border border-zinc-300 dark:border-zinc-700 disabled:opacity-50"
        >
          {testing ? t("common.loading") : t("settings.test")}
        </button>
      </div>

      {testResult && (
        <div
          className={`text-sm px-3 py-2 rounded-md ${
            testResult.ok
              ? "bg-emerald-50 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-300"
              : "bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-300"
          }`}
        >
          [{testResult.ok ? t("settings.test_ok") : t("settings.test_fail")}]{" "}
          {testResult.message}
        </div>
      )}
    </div>
  );
}
