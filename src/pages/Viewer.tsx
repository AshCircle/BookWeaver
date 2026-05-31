import { useEffect, useMemo, useState } from "react";
import { useParams, useSearchParams } from "react-router-dom";
import { ChapterTree } from "@/components/ChapterTree";
import { DensitySwitch } from "@/components/DensitySwitch";
import { SourceBadges } from "@/components/SourceBadges";
import { useT } from "@/lib/i18n";
import {
  getBook,
  getChapters,
  getChapterSources,
  getSources,
} from "@/lib/tauri";
import type { Book, Chapter, Density, Source } from "@/lib/types";

export default function ViewerPage() {
  const { t } = useT();
  const params = useParams<{ id: string }>();
  const bookId = Number(params.id);
  const [searchParams, setSearchParams] = useSearchParams();
  const density = (searchParams.get("density") as Density | null) ?? "medium";

  const [book, setBook] = useState<Book | null>(null);
  const [chapters, setChapters] = useState<Chapter[]>([]);
  const [activeId, setActiveId] = useState<number | null>(null);
  const [sources, setSources] = useState<Source[]>([]);
  const [allSources, setAllSources] = useState<Source[]>([]);

  useEffect(() => {
    if (!bookId) return;
    let cancelled = false;
    Promise.all([getBook(bookId), getChapters(bookId), getSources(bookId)])
      .then(([b, ch, src]) => {
        if (cancelled) return;
        setBook(b);
        setChapters(ch);
        setAllSources(src);
        if (ch.length > 0) {
          setActiveId(ch[0].id);
        }
      })
      .catch(console.error);
    return () => {
      cancelled = true;
    };
  }, [bookId]);

  useEffect(() => {
    if (activeId == null) {
      setSources([]);
      return;
    }
    let cancelled = false;
    getChapterSources(activeId)
      .then((next) => {
        if (!cancelled) setSources(next);
      })
      .catch(console.error);
    return () => {
      cancelled = true;
    };
  }, [activeId]);

  const activeChapter = useMemo(
    () => chapters.find((c) => c.id === activeId) ?? null,
    [chapters, activeId],
  );

  function setDensity(d: Density) {
    const next = new URLSearchParams(searchParams);
    next.set("density", d);
    setSearchParams(next, { replace: true });
  }

  const summary =
    activeChapter == null
      ? null
      : density === "light"
        ? activeChapter.summary_light
        : density === "deep"
          ? activeChapter.summary_deep
          : activeChapter.summary_med;

  return (
    <div className="grid grid-cols-12 gap-0 min-h-[calc(100vh-3.5rem)]">
      <aside className="col-span-3 border-r border-zinc-200 dark:border-zinc-800 bg-zinc-50/50 dark:bg-zinc-900/40">
        <div className="px-3 py-3 border-b border-zinc-200 dark:border-zinc-800">
          <div className="text-xs text-zinc-500">{book?.author ?? "—"}</div>
          <div className="font-semibold truncate">{book?.title ?? "—"}</div>
        </div>
        <ChapterTree
          chapters={chapters}
          activeId={activeId}
          onSelect={setActiveId}
        />
      </aside>

      <main className="col-span-6 px-8 py-6">
        <div className="flex items-center justify-between mb-4">
          <h1 className="text-xl font-semibold">
            {activeChapter?.title ?? t("viewer.no_chapter")}
          </h1>
          <DensitySwitch value={density} onChange={setDensity} />
        </div>
        {summary?.startsWith("[TODO: synthesis 미구현]") && (
          <div className="mb-4 px-3 py-2 rounded-md bg-amber-50 dark:bg-amber-900/20 text-amber-700 dark:text-amber-300 text-sm">
            {t("viewer.todo_synthesis")}
          </div>
        )}
        <article className="prose prose-zinc dark:prose-invert max-w-none whitespace-pre-wrap leading-relaxed">
          {summary ?? t("viewer.no_chapter")}
        </article>
      </main>

      <aside className="col-span-3 border-l border-zinc-200 dark:border-zinc-800 bg-zinc-50/50 dark:bg-zinc-900/40">
        <div className="px-3 py-3 border-b border-zinc-200 dark:border-zinc-800 text-sm font-medium">
          {t("viewer.sources")}{" "}
          <span className="text-zinc-500 font-normal">
            ({activeId == null ? allSources.length : sources.length})
          </span>
        </div>
        <SourceBadges sources={activeId == null ? allSources : sources} />
      </aside>
    </div>
  );
}
