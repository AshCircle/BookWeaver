import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useT } from "@/lib/i18n";
import { startBook } from "@/lib/tauri";

export default function SearchPage() {
  const { t } = useT();
  const navigate = useNavigate();
  const [title, setTitle] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    if (!title.trim() || busy) return;
    setBusy(true);
    setError(null);
    try {
      await startBook(title.trim());
      navigate("/library");
    } catch (err) {
      const message =
        typeof err === "object" && err !== null && "message" in err
          ? String((err as { message: string }).message)
          : String(err);
      setError(message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="max-w-2xl mx-auto px-6 py-16">
      <h1 className="text-3xl font-bold mb-2">BookWeaver</h1>
      <p className="text-sm text-zinc-500 mb-8">{t("search.hint")}</p>
      <form onSubmit={submit} className="space-y-4">
        <input
          autoFocus
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder={t("search.placeholder")}
          className="w-full px-4 py-3 rounded-lg border border-zinc-300 dark:border-zinc-700 bg-white dark:bg-zinc-900 outline-none focus:border-blue-500"
        />
        <button
          type="submit"
          disabled={busy || !title.trim()}
          className="px-5 py-2.5 rounded-lg bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 disabled:opacity-50"
        >
          {busy ? t("common.loading") : t("search.start")}
        </button>
      </form>
      {error && (
        <div className="mt-4 text-sm text-red-600 dark:text-red-400 whitespace-pre-wrap">
          {error}
        </div>
      )}
    </div>
  );
}
