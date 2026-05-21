import type { Chapter } from "@/lib/types";

interface Props {
  chapters: Chapter[];
  activeId: number | null;
  onSelect: (id: number) => void;
}

export function ChapterTree({ chapters, activeId, onSelect }: Props) {
  if (chapters.length === 0) {
    return (
      <div className="text-sm text-zinc-500 px-3 py-4">
        챕터가 아직 생성되지 않았습니다.
      </div>
    );
  }

  return (
    <ul className="py-2 text-sm">
      {chapters.map((ch) => {
        const active = ch.id === activeId;
        return (
          <li key={ch.id}>
            <button
              type="button"
              onClick={() => onSelect(ch.id)}
              className={`w-full text-left px-3 py-2 rounded-md transition-colors ${
                active
                  ? "bg-zinc-200 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 font-medium"
                  : "hover:bg-zinc-100 dark:hover:bg-zinc-800/60"
              }`}
            >
              <span className="text-zinc-400 mr-2">
                {String(ch.order_idx + 1).padStart(2, "0")}.
              </span>
              {ch.title}
            </button>
          </li>
        );
      })}
    </ul>
  );
}
