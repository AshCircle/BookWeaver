import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { ProgressTimeline } from "@/components/ProgressTimeline";
import { useT } from "@/lib/i18n";
import {
  listBooks,
  onProgress,
  onBookReady,
  onBookFailed,
} from "@/lib/tauri";
import type { Book, ProgressEvent, Stage } from "@/lib/types";

interface ProgressState {
  stage: Stage | null;
  status: "started" | "done" | "failed" | "idle";
}

export default function LibraryPage() {
  const { t } = useT();
  const [books, setBooks] = useState<Book[]>([]);
  const [progress, setProgress] = useState<Record<number, ProgressState>>({});
  const [loading, setLoading] = useState(true);

  async function refresh() {
    try {
      const list = await listBooks();
      setBooks(list);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    refresh();
    let disposed = false;
    const unlistens: Array<() => void> = [];
    const register = (p: Promise<() => void>) => {
      p.then((un) => {
        if (disposed) {
          un();
          return;
        }
        unlistens.push(un);
      }).catch(console.error);
    };

    register(
      onProgress((evt: ProgressEvent) => {
        setProgress((prev) => ({
          ...prev,
          [evt.book_id]: { stage: evt.stage, status: evt.status },
        }));
      }),
    );

    register(
      onBookReady(() => {
        refresh();
      }),
    );

    register(
      onBookFailed(() => {
        refresh();
      }),
    );

    return () => {
      disposed = true;
      unlistens.forEach((un) => un());
    };
  }, []);

  if (loading) {
    return <div className="p-6 text-zinc-500">{t("common.loading")}</div>;
  }

  if (books.length === 0) {
    return (
      <div className="p-8 text-center">
        <p className="text-zinc-500 mb-4">{t("library.empty")}</p>
        <Link
          to="/"
          className="inline-block px-4 py-2 rounded-lg bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900"
        >
          {t("nav.search")}
        </Link>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto px-6 py-8">
      <h1 className="text-2xl font-semibold mb-6">{t("nav.library")}</h1>
      <ul className="space-y-3">
        {books.map((book) => {
          const p = progress[book.id] ?? { stage: null, status: "idle" as const };
          const ready = book.status === "ready";
          return (
            <li
              key={book.id}
              className="border border-zinc-200 dark:border-zinc-800 rounded-lg p-4 bg-white dark:bg-zinc-900"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <div className="font-medium">{book.title}</div>
                  <div className="text-xs text-zinc-500 mt-0.5">
                    {book.author ?? "—"} · {book.language ?? "—"} ·{" "}
                    {t(`status.${book.status}` as never)}
                  </div>
                  {book.status === "failed" && book.error_message && (
                    <div className="mt-1 text-xs text-red-500 line-clamp-2">
                      {book.error_message}
                    </div>
                  )}
                </div>
                {ready && (
                  <Link
                    to={`/book/${book.id}`}
                    className="shrink-0 px-3 py-1.5 text-sm rounded-md bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900"
                  >
                    {t("library.open")}
                  </Link>
                )}
              </div>
              <div className="mt-3">
                <ProgressTimeline
                  currentStage={p.stage}
                  status={p.status}
                  bookStatus={book.status}
                />
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
