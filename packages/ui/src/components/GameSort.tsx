import { useTranslation } from "../i18n";
import type { GameSummary } from "../types";

export type GameSortOrder = "default" | "release-newest" | "release-oldest";

export function sortGamesByRelease(games: GameSummary[], order: GameSortOrder): GameSummary[] {
  if (order === "default") return games;

  const direction = order === "release-newest" ? -1 : 1;
  return [...games].sort((a, b) => {
    if (!a.originalReleaseDate) return b.originalReleaseDate ? 1 : 0;
    if (!b.originalReleaseDate) return -1;
    return direction * a.originalReleaseDate.localeCompare(b.originalReleaseDate);
  });
}

export function GameSort({
  scope,
  order,
  onChange,
}: {
  scope: string;
  order: GameSortOrder;
  onChange: (order: GameSortOrder) => void;
}) {
  const { t } = useTranslation();

  return (
    <select
      className="kt-game-sort"
      aria-label={t("Sort games in {0}", [scope])}
      value={order}
      onChange={(event) => onChange(event.target.value as GameSortOrder)}
    >
      <option value="default">{t("Sort by: Default")}</option>
      <option value="release-newest">{t("Release date (newest first)")}</option>
      <option value="release-oldest">{t("Release date (oldest first)")}</option>
    </select>
  );
}
