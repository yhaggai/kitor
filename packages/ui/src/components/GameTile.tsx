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
}: {
  game: GameSummary;
  onOpen: (game: GameSummary, element: HTMLButtonElement) => void;
  highlighted?: boolean;
  showStatus?: boolean;
}) {
  const { t, language } = useTranslation();
  const title = gameTitle(game, language);
  return (
    <button
      type="button"
      className={`kt-game-tile ${highlighted ? "kt-game-tile--highlighted" : ""}`}
      data-game-tile={game.id}
      aria-label={`${t("Explore game")} ${title}${showStatus ? ` ${t(statusLabels[game.status])}` : ""}`}
      onClick={(event) => onOpen(game, event.currentTarget)}
    >
      <GameCover game={game} />
      <span className="kt-game-title" dir="auto">
        {title}
      </span>
      {showStatus && <StatusBadge status={game.status} />}
    </button>
  );
}
