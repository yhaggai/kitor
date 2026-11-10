import { Home, Library, Settings } from "lucide-react";
import type { ReactNode } from "react";
import { useTranslation } from "../i18n";
import type { Collection } from "../types";
import { BrandLogo } from "./BrandLogo";

export function AppShell({
  active,
  onNavigate,
  children,
  footer,
}: {
  active: Collection | "settings";
  onNavigate: (view: Collection | "settings") => void;
  children: ReactNode;
  footer?: ReactNode;
}) {
  const { t } = useTranslation();
  return (
    <div className="kitor-theme kt-shell">
      <a className="kt-skip-link" href="#kitor-main">
        {t("Skip to content")}{" "}
      </a>
      <aside className="kt-sidebar">
        <div className="kt-brand">
          <BrandLogo />
        </div>
        <nav aria-label={t("Main navigation")}>
          {(
            [
              { id: "discovery", label: t("Home"), icon: Home },
              { id: "library", label: t("My Games"), icon: Library },
            ] as const
          ).map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              type="button"
              aria-current={active === id ? "page" : undefined}
              onClick={() => onNavigate(id)}
            >
              <Icon size={20} />
              <span>{label}</span>
            </button>
          ))}
        </nav>
        <div className="kt-sidebar-bottom">
          <button
            type="button"
            aria-current={active === "settings" ? "page" : undefined}
            onClick={() => onNavigate("settings")}
          >
            <Settings size={20} />
            <span>{t("Settings")}</span>
          </button>
          {footer}
        </div>
      </aside>
      <main id="kitor-main" className="kt-main" tabIndex={-1}>
        {children}
      </main>
    </div>
  );
}
