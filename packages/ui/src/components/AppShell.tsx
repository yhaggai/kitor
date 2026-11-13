import { type ComponentType, type ReactNode, type Ref, useRef } from "react";
import { useTranslation } from "../i18n";
import { type AnimatedIconHandle, HomeIcon, LibraryIcon, SettingsIcon } from "../icons";
import type { Collection } from "../types";
import { BrandLogo } from "./BrandLogo";

type NavIcon = ComponentType<{
  ref?: Ref<AnimatedIconHandle>;
  size?: number;
}>;

function NavButton({
  label,
  icon: Icon,
  current,
  onClick,
}: {
  label: string;
  icon: NavIcon;
  current?: boolean;
  onClick: () => void;
}) {
  const iconRef = useRef<AnimatedIconHandle>(null);
  return (
    <button
      type="button"
      aria-current={current ? "page" : undefined}
      onClick={onClick}
      onMouseEnter={() => iconRef.current?.startAnimation()}
      onMouseLeave={() => iconRef.current?.stopAnimation()}
    >
      <Icon ref={iconRef} size={20} />
      <span>{label}</span>
    </button>
  );
}

export function AppShell({
  active,
  onNavigate,
  children,
  footer,
  account,
}: {
  active: Collection | "settings";
  onNavigate: (view: Collection | "settings") => void;
  children: ReactNode;
  footer?: ReactNode;
  account?: ReactNode;
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
        <nav aria-label={t("Main navigation")} data-active={active}>
          <NavButton
            label={t("Home")}
            icon={HomeIcon}
            current={active === "discovery"}
            onClick={() => onNavigate("discovery")}
          />
          <NavButton
            label={t("My Games")}
            icon={LibraryIcon}
            current={active === "library"}
            onClick={() => onNavigate("library")}
          />
        </nav>
        <div className="kt-sidebar-bottom">
          <NavButton
            label={t("Settings")}
            icon={SettingsIcon}
            current={active === "settings"}
            onClick={() => onNavigate("settings")}
          />
          {footer}
        </div>
      </aside>
      <main id="kitor-main" className="kt-main" tabIndex={-1}>
        {account}
        {children}
      </main>
    </div>
  );
}
