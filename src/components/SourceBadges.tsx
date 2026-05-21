import { useState } from "react";
import { open as openShell } from "@tauri-apps/plugin-shell";
import { useT } from "@/lib/i18n";
import type { Source } from "@/lib/types";

interface Props {
  sources: Source[];
}

export function SourceBadges({ sources }: Props) {
  const { t } = useT();
  const [openId, setOpenId] = useState<number | null>(null);

  if (sources.length === 0) {
    return (
      <div className="text-sm text-zinc-500 px-3 py-4">출처가 없습니다.</div>
    );
  }

  return (
    <ul className="space-y-2 px-3 py-3">
      {sources.map((src) => {
        const opened = openId === src.id;
        return (
          <li key={src.id} className="border border-zinc-200 dark:border-zinc-800 rounded-md p-2">
            <button
              type="button"
              onClick={() => setOpenId(opened ? null : src.id)}
              className="w-full text-left"
            >
              <div className="text-xs text-zinc-400">{src.site ?? "—"}</div>
              <div className="text-sm font-medium line-clamp-2">
                {src.title ?? src.url}
              </div>
            </button>
            {opened && (
              <div className="mt-2 text-xs space-y-1">
                <div className="text-zinc-500 break-all">{src.url}</div>
                <button
                  type="button"
                  className="text-blue-600 dark:text-blue-400 underline"
                  onClick={() => {
                    openShell(src.url).catch(console.error);
                  }}
                >
                  {t("viewer.open_source")}
                </button>
              </div>
            )}
          </li>
        );
      })}
    </ul>
  );
}
