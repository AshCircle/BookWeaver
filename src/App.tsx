import { useEffect, useMemo, useState } from "react";
import {
  Link,
  NavLink,
  Route,
  Routes,
  useLocation,
} from "react-router-dom";
import {
  I18nContext,
  STORAGE_LANG_KEY,
  translate,
  useT,
  type DictKey,
} from "@/lib/i18n";
import SearchPage from "@/pages/Search";
import LibraryPage from "@/pages/Library";
import ViewerPage from "@/pages/Viewer";
import SettingsPage from "@/pages/Settings";

type Lang = "ko" | "en";

function getInitialLang(): Lang {
  if (typeof localStorage !== "undefined") {
    const v = localStorage.getItem(STORAGE_LANG_KEY);
    if (v === "en" || v === "ko") return v;
  }
  return "ko";
}

export default function App() {
  const [lang, setLangState] = useState<Lang>(getInitialLang);

  function setLang(next: Lang) {
    setLangState(next);
    try {
      localStorage.setItem(STORAGE_LANG_KEY, next);
    } catch {
      // ignore
    }
  }

  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  const i18n = useMemo(
    () => ({
      lang,
      setLang,
      t: (key: DictKey) => translate(lang, key),
    }),
    [lang],
  );

  return (
    <I18nContext.Provider value={i18n}>
      <div className="min-h-full flex flex-col">
        <NavBar />
        <div className="flex-1">
          <Routes>
            <Route path="/" element={<SearchPage />} />
            <Route path="/library" element={<LibraryPage />} />
            <Route path="/book/:id" element={<ViewerPage />} />
            <Route path="/settings" element={<SettingsPage />} />
            <Route path="*" element={<NotFound />} />
          </Routes>
        </div>
      </div>
    </I18nContext.Provider>
  );
}

function NavBar() {
  const { t } = useT();
  const location = useLocation();
  const isViewer = location.pathname.startsWith("/book/");
  return (
    <nav
      className={`flex items-center justify-between border-b border-zinc-200 dark:border-zinc-800 bg-white/80 dark:bg-zinc-950/80 backdrop-blur ${
        isViewer ? "px-4 h-12" : "px-6 h-14"
      }`}
    >
      <Link to="/" className="font-semibold">
        BookWeaver
      </Link>
      <div className="flex items-center gap-1 text-sm">
        <NavItem to="/" label={t("nav.search")} />
        <NavItem to="/library" label={t("nav.library")} />
        <NavItem to="/settings" label={t("nav.settings")} />
      </div>
    </nav>
  );
}

function NavItem({ to, label }: { to: string; label: string }) {
  return (
    <NavLink
      to={to}
      end
      className={({ isActive }) =>
        `px-3 py-1.5 rounded-md transition-colors ${
          isActive
            ? "bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
            : "text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-100"
        }`
      }
    >
      {label}
    </NavLink>
  );
}

function NotFound() {
  return (
    <div className="p-8 text-zinc-500">
      <p>찾을 수 없는 페이지입니다.</p>
    </div>
  );
}
