import { useT } from "@/lib/i18n";
import type { Density } from "@/lib/types";

interface Props {
  value: Density;
  onChange: (d: Density) => void;
}

const ORDER: Density[] = ["light", "medium", "deep"];

export function DensitySwitch({ value, onChange }: Props) {
  const { t } = useT();
  return (
    <div className="inline-flex bg-zinc-100 dark:bg-zinc-800 rounded-md p-0.5 text-xs">
      {ORDER.map((d) => {
        const active = d === value;
        return (
          <button
            key={d}
            type="button"
            onClick={() => onChange(d)}
            className={`px-3 py-1 rounded-md transition-colors ${
              active
                ? "bg-white dark:bg-zinc-700 shadow-sm font-medium"
                : "text-zinc-500 hover:text-zinc-800 dark:hover:text-zinc-200"
            }`}
          >
            {t(`density.${d}` as never)}
          </button>
        );
      })}
    </div>
  );
}
