import {
  Bug,
  Download,
  ExternalLink,
  FolderOpen,
  Gamepad2,
  KeyRound,
  LogIn,
  LogOut,
  type LucideIcon,
  Monitor,
  RefreshCw,
  Shield,
  Trash2,
} from "lucide-react";
import type { ReactNode } from "react";
import kitorIcon from "../assets/kitor-logo.svg";
import { Button } from "../components/Button";
import { ManageMenu } from "../components/ManageMenu";
import { useTranslation } from "../i18n";

const actionIcons: Record<string, LucideIcon> = {
  "check-app": RefreshCw,
  "connect-steam": LogIn,
  "disconnect-steam": LogOut,
  "refresh-steam": RefreshCw,
  "steam-key": KeyRound,
  "steam-privacy": Shield,
  "steam-api": ExternalLink,
  "connect-gog": LogIn,
  "disconnect-gog": LogOut,
  "refresh-gog": RefreshCw,
  "open-runtime": ExternalLink,
  "runtime-folder": FolderOpen,
  "update-runtime": RefreshCw,
  "remove-runtime": Trash2,
  "install-runtime": Download,
  report: Bug,
};

function SettingsCard({
  title,
  description,
  detail,
  icon,
  status,
  children,
}: {
  title: string;
  description: string;
  detail?: string;
  icon: ReactNode;
  status?: "ready" | "attention" | "unavailable";
  children: ReactNode;
}) {
  const { t } = useTranslation();
  return (
    <section className="kt-action-card kt-settings-card">
      <div className="kt-settings-card-header">
        <div className="kt-provider-icon" aria-hidden="true">
          {icon}
        </div>
        <div className="kt-settings-card-copy">
          <h2 className="kt-settings-card-title">
            {title}
            {status && (
              <span
                className={`kt-integration-dot kt-integration-dot--${status}`}
                aria-hidden="true"
              />
            )}
          </h2>
          <p>{description}</p>
        </div>
        <ManageMenu label={t("Manage {0}", [title])}>{children}</ManageMenu>
      </div>
      {detail && <p className="kt-settings-card-detail">{detail}</p>}
    </section>
  );
}

export type SettingsState = {
  steam: boolean;
  gog: boolean;
  steamKey: boolean;
  runtime: boolean;
  runtimeUpdated: boolean;
  appUpdated: boolean;
};
export function SettingsView({
  state,
  onAction,
}: {
  state: SettingsState;
  onAction: (action: string) => void;
}) {
  const { t } = useTranslation();
  const buttons = (items: [string, string][]) => (
    <div className="kt-flow-actions">
      {items.map(([id, label]) => {
        const Icon = actionIcons[id];
        return (
          <Button key={id} variant="secondary" onClick={() => onAction(id)}>
            {Icon && <Icon size={18} aria-hidden="true" className="kt-action-icon" />}
            {label}
          </Button>
        );
      })}
    </div>
  );
  return (
    <section>
      <p className="kt-eyebrow">{t("MAKE YOURSELF AT HOME")}</p>
      <h1>{t("Settings")}</h1>
      <p>{t("Accounts, game runtimes and Kitor updates.")}</p>
      <div className="kt-settings-grid">
        <SettingsCard
          title={t("Kitor updates")}
          icon={<img src={kitorIcon} alt="" />}
          description={
            state.appUpdated ? t("Kitor is up to date.") : "Installed: 1.0 · Review build"
          }
        >
          {buttons([["check-app", "Check for app updates"]])}
        </SettingsCard>
        <SettingsCard
          title="Steam"
          status={!state.steam ? "unavailable" : state.steamKey ? "ready" : "attention"}
          icon={<Gamepad2 aria-hidden="true" />}
          detail="Sign in on Steam’s website. Your game details must be public for library lookup."
          description={
            state.steam
              ? t("Connected · {0}", [
                  state.steamKey ? "12 games in your library" : "Add an API key to check ownership",
                ])
              : t("Connect Steam to find games you own.")
          }
        >
          {buttons([
            [
              state.steam ? "disconnect-steam" : "connect-steam",
              state.steam ? t("Disconnect Steam") : t("Connect Steam"),
            ],
            ...(state.steam && state.steamKey
              ? [["refresh-steam", "Refresh Steam library"] as [string, string]]
              : []),
            ["steam-key", state.steamKey ? t("Manage Steam API key") : t("Add Steam API key")],
            ["steam-privacy", "Steam privacy settings"],
            ["steam-api", "Get a Steam API key"],
          ])}
        </SettingsCard>
        <SettingsCard
          title="GOG"
          status={state.gog ? "ready" : "unavailable"}
          icon={<Gamepad2 aria-hidden="true" />}
          description={
            state.gog
              ? "Connected · 8 games in your library"
              : t("Connect your GOG account to download games you own.")
          }
        >
          {buttons([
            [
              state.gog ? "disconnect-gog" : "connect-gog",
              state.gog ? t("Log out of GOG") : t("Connect GOG"),
            ],
            ...(state.gog ? [["refresh-gog", "Refresh GOG library"] as [string, string]] : []),
          ])}
        </SettingsCard>
        <SettingsCard
          title="ScummVM"
          status={!state.runtime ? "unavailable" : state.runtimeUpdated ? "ready" : "attention"}
          icon={<Monitor aria-hidden="true" />}
          description={
            state.runtime
              ? t("Ready · {0}", [
                  state.runtimeUpdated ? t("latest version") : t("update available"),
                ])
              : t("Not downloaded · required for ScummVM games.")
          }
        >
          {buttons(
            state.runtime
              ? [
                  ["open-runtime", "Open ScummVM"],
                  ["runtime-folder", "Open ScummVM folder"],
                  ...(!state.runtimeUpdated
                    ? [["update-runtime", "Update ScummVM"] as [string, string]]
                    : []),
                  ["remove-runtime", "Uninstall ScummVM"],
                ]
              : [["install-runtime", "Download ScummVM"]]
          )}
        </SettingsCard>
        <SettingsCard
          icon={<img src={kitorIcon} alt="" />}
          title={t("About Kitor")}
          description="Manage games and community translations."
        >
          {buttons([["report", "Report a bug"]])}
        </SettingsCard>
      </div>
    </section>
  );
}
