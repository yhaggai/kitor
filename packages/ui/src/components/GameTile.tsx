import { useTranslation } from "../i18n";
import type { GameSummary } from "../types";
import { GameCover } from "./GameCover";
import { StatusBadge, statusLabels } from "./StatusBadge";

export function GameTile({
  game,
  onOpen,
  highlighted = false,
}: {
  game: GameSummary;
  onOpen: (game: GameSummary, element: HTMLButtonElement) => void;
  highlighted?: boolean;
}) {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className={`kt-game-tile ${highlighted ? "kt-game-tile--highlighted" : ""}`}
      data-game-tile={game.id}
      aria-label={`${t("Explore game")} ${game.title} ${t(statusLabels[game.status])}`}
      onClick={(event) => onOpen(game, event.currentTarget)}
    >
      <GameCover game={game} />
      <span className="kt-game-title" dir="auto">
        {game.title}
      </span>
      <StatusBadge status={game.status} />
    </button>
  );
}
