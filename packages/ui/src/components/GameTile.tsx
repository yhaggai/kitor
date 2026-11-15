import { gameTitle } from "../hebrewTitle";
import { useTranslation } from "../i18n";
import type { GameSummary } from "../types";
import { GameCover } from "./GameCover";
import { StatusBadge, statusLabels } from "./StatusBadge";

export function GameTile({
  game,
  onOpen,
  highlighted = false,
  showStatus = false,
  showBeta = false,
}: {
  game: GameSummary;
  onOpen: (game: GameSummary, element: HTMLButtonElement) => void;
  highlighted?: boolean;
  showStatus?: boolean;
  showBeta?: boolean;
}) {
  const { t, language } = useTranslation();
  const title = gameTitle(game, language);
  const beta = showBeta ? t("Beta") : undefined;
  return (
    <button
      type="button"
      className={`kt-game-tile ${highlighted ? "kt-game-tile--highlighted" : ""}`}
      data-game-tile={game.id}
      aria-label={`${t("Explore game")} ${title}${showStatus ? ` ${t(statusLabels[game.status])}` : ""}${beta ? `, ${beta}` : ""}`}
      onClick={(event) => onOpen(game, event.currentTarget)}
    >
      <GameCover game={game} />
      {beta && (
        <span className="kt-beta-corner" aria-hidden="true">
          <span>{beta}</span>
        </span>
      )}
      <span className="kt-game-title" dir="auto">
        {title}
      </span>
      {showStatus && <StatusBadge status={game.status} />}
    </button>
  );
}
