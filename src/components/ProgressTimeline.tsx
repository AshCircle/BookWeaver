import { useT } from "@/lib/i18n";
import type { Stage } from "@/lib/types";

const STAGES: Stage[] = [
  "search",
  "collect",
  "structure",
  "cluster",
  "synthesis",
  "ready",
];

interface Props {
  currentStage: Stage | null;
  status: "started" | "done" | "failed" | "idle";
  bookStatus?: string;
}

export function ProgressTimeline({ currentStage, status, bookStatus }: Props) {
  const { t } = useT();
  const stageIndex = currentStage ? STAGES.indexOf(currentStage) : -1;
  const failed = status === "failed" || bookStatus === "failed";
  const ready = bookStatus === "ready";

  return (
    <ol className="flex items-center gap-1 text-xs">
      {STAGES.map((stage, idx) => {
        const reached = ready || (stageIndex >= idx);
        const active =
          !ready &&
          !failed &&
          stageIndex === idx &&
          status === "started";
        const tone = failed && stageIndex === idx
          ? "bg-red-500 text-white"
          : ready || (reached && stageIndex > idx)
            ? "bg-emerald-500 text-white"
            : active
              ? "bg-amber-400 text-amber-950 animate-pulse"
              : reached
                ? "bg-emerald-500 text-white"
                : "bg-zinc-200 dark:bg-zinc-700 text-zinc-500";
        return (
          <li key={stage} className="flex items-center gap-1">
            <span
              className={`px-2 py-0.5 rounded-full whitespace-nowrap ${tone}`}
            >
              {t(`stage.${stage}` as never)}
            </span>
            {idx < STAGES.length - 1 && (
              <span className="text-zinc-400">›</span>
            )}
          </li>
        );
      })}
    </ol>
  );
}
