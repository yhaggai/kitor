import { ArrowLeft, Expand } from "lucide-react";
import { type ReactNode, useId, useState } from "react";
import { Button } from "../components/Button";
import { FlowDialog } from "../components/FlowDialog";
import { GameBanner } from "../components/GameBanner";
import { GameBox } from "../components/GameBox";
import { GameCover } from "../components/GameCover";
import { GameDescription } from "../components/GameDescription";
import { GameGallery } from "../components/GameGallery";
import { ManageMenu } from "../components/ManageMenu";
import { StatusBadge } from "../components/StatusBadge";
import { useTranslation } from "../i18n";
import type { Collection, GameSummary } from "../types";

export function GameDetailsView({
  game,
  collection,
  onBack,
  actions,
  manage,
  manageControl,
  translation,
  descriptionCredit,
}: {
  game: GameSummary;
  collection: Collection;
  onBack: () => void;
  actions?: ReactNode;
  manage?: ReactNode;
  manageControl?: ReactNode;
  translation?: ReactNode;
  descriptionCredit?: ReactNode;
}) {
  const { t, language } = useTranslation();
  const [tab, setTab] = useState("overview");
  const [boxOpen, setBoxOpen] = useState(false);
  const id = useId();
  const tabs = ["overview", "translation"];
  return (
    <section className="kt-details" aria-label={t("{0} details", [game.title])}>
      <Button variant="ghost" onClick={onBack}>
        <ArrowLeft size={18} />
        {t("Back to")} {collection === "library" ? t("library") : t("discovery")}
      </Button>
      <GameBanner game={game} />
      <div className="kt-game-command-bar">
        <div className="kt-flow-actions">{actions}</div>
        <StatusBadge status={game.status} />
        {manage && <ManageMenu>{manage}</ManageMenu>}
        {manageControl}
      </div>
      <div className="kt-detail-hero kt-detail-hero--banner">
        <div className="kt-detail-art">
          <button
            type="button"
            className="kt-box-preview"
            aria-label={t("Inspect {0} game box", [game.title])}
            aria-haspopup="dialog"
            title={t("Click to inspect game box")}
            onClick={() => setBoxOpen(true)}
          >
            <GameCover game={game} />
            <span className="kt-box-expand" aria-hidden="true">
              <Expand size={16} />
            </span>
          </button>
        </div>
        <div className="kt-detail-copy">
          <p className="kt-eyebrow">
            {collection === "library"
              ? t("FROM YOUR COLLECTION")
              : t("DISCOVER SOMETHING WONDERFUL")}
          </p>
          <h1 data-detail-heading tabIndex={-1} dir="auto">
            {game.title}
          </h1>
          <StatusBadge status={game.status} />
          <GameDescription
            key={`${game.id}-${game.description}`}
            description={game.description || "הרפתקה חדשה מחכה לכם."}
          />
          <dl className="kt-facts">
            <div>
              <dt>{t("Translation")}</dt>
              <dd>{game.language || t("Not available")}</dd>
            </div>
            <div>
              <dt>{t("Version")}</dt>
              <dd>{game.version || "—"}</dd>
            </div>
          </dl>
        </div>
      </div>
      {boxOpen && (
        <FlowDialog title={t("{0} · Game box", [game.title])} onClose={() => setBoxOpen(false)}>
          <div className="kt-box-modal-viewer">
            <GameBox key={game.id} game={game} />
          </div>
        </FlowDialog>
      )}
      <GameGallery key={game.id} title={game.title} screenshots={game.screenshots} />
      <div className="kt-detail-tabs" role="tablist" aria-label={t("Game information")}>
        {tabs.map((name, index) => (
          <button
            key={t(name)}
            id={`${id}-${name}`}
            type="button"
            role="tab"
            aria-selected={tab === name}
            aria-controls={`${id}-panel`}
            tabIndex={tab === name ? 0 : -1}
            onClick={() => setTab(name)}
            onKeyDown={(event) => {
              const next =
                event.key === (language === "he" ? "ArrowLeft" : "ArrowRight")
                  ? (index + 1) % tabs.length
                  : event.key === (language === "he" ? "ArrowRight" : "ArrowLeft")
                    ? (index + tabs.length - 1) % tabs.length
                    : event.key === "Home"
                      ? 0
                      : event.key === "End"
                        ? tabs.length - 1
                        : -1;
              if (next >= 0) {
                event.preventDefault();
                setTab(tabs[next]);
                document.getElementById(`${id}-${tabs[next]}`)?.focus();
              }
            }}
          >
            {t(name)}
          </button>
        ))}
      </div>
      <div
        className="kt-detail-panel"
        id={`${id}-panel`}
        role="tabpanel"
        aria-labelledby={`${id}-${tab}`}
        // biome-ignore lint/a11y/noNoninteractiveTabindex: Text-only tab panels need a keyboard focus target.
        tabIndex={0}
      >
        {tab === "overview" ? (
          <>
            <h2 dir="rtl" lang="he">
              על המשחק
            </h2>
            <p dir="rtl" lang="he" style={{ whiteSpace: "pre-line" }}>
              {game.description || "הרפתקה חדשה מחכה לכם."}
            </p>
            {descriptionCredit}
          </>
        ) : tab === "translation" ? (
          <>
            <h2>{t("Community translation")}</h2>
            <StatusBadge status={game.status} />
            <p>
              {t("Language:")} {game.language || t("Not available")}
            </p>
            {game.author && (
              <p dir="auto">
                {t("Translation by")} {game.author}
              </p>
            )}
            {game.version && (
              <p>
                {t("Version")} {game.version}
              </p>
            )}
            {descriptionCredit}
            {translation}
          </>
        ) : (
          manage
        )}
      </div>
    </section>
  );
}
