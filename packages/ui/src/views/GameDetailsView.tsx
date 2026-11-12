import { ArrowLeft, Expand } from "lucide-react";
import { type ReactNode, useId, useState } from "react";
import { Button } from "../components/Button";
import { FlowDialog } from "../components/FlowDialog";
import { GameBanner } from "../components/GameBanner";
import { GameBox } from "../components/GameBox";
import { GameCover } from "../components/GameCover";
import { GameGallery } from "../components/GameGallery";
import { ManageMenu } from "../components/ManageMenu";
import { gameTitle } from "../hebrewTitle";
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
  const [boxOpen, setBoxOpen] = useState(false);
  const id = useId();
  const title = gameTitle(game, language);
  const titleIsHebrew = /[\u0590-\u05FF]/.test(title);
  return (
    <section className="kt-details" aria-label={t("{0} details", [title])}>
      <Button variant="ghost" onClick={onBack}>
        <ArrowLeft size={18} />
        {t("Back to")} {collection === "library" ? t("library") : t("discovery")}
      </Button>
      <GameBanner game={game} />
      <div className="kt-detail-hero kt-detail-hero--banner">
        <div className="kt-detail-art">
          <button
            type="button"
            className="kt-box-preview"
            aria-label={t("Inspect {0} game box", [title])}
            aria-haspopup="dialog"
            title={t("Click to inspect game box")}
            onClick={() => setBoxOpen(true)}
          >
            <GameCover game={game} />
            <span className="kt-box-expand" aria-hidden="true">
              <Expand size={16} />
            </span>
          </button>
          {actions && <div className="kt-primary-actions">{actions}</div>}
        </div>
        <div className="kt-detail-copy">
          <div className="kt-detail-title-row">
            <h1
              data-detail-heading
              tabIndex={-1}
              dir={titleIsHebrew ? "rtl" : "auto"}
              lang={titleIsHebrew ? "he" : undefined}
            >
              {title}
            </h1>
            {manageControl}
            {manage && <ManageMenu>{manage}</ManageMenu>}
          </div>
          <div className="kt-detail-description">
            <p dir="rtl" lang="he">
              {game.description || "הרפתקה חדשה מחכה לכם."}
            </p>
            {descriptionCredit}
          </div>
        </div>
      </div>
      <GameGallery key={game.id} title={title} screenshots={game.screenshots} />
      {boxOpen && (
        <FlowDialog title={t("{0} · Game box", [title])} onClose={() => setBoxOpen(false)}>
          <div className="kt-box-modal-viewer">
            <GameBox key={game.id} game={game} />
          </div>
        </FlowDialog>
      )}
      <section className="kt-detail-panel" aria-labelledby={`${id}-translation`}>
        <h2 id={`${id}-translation`}>{t("Community translation")}</h2>
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
        {translation}
      </section>
    </section>
  );
}
