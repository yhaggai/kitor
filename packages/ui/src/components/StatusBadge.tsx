import { useTranslation } from "../i18n";
import type { GameStatus } from "../types";

export const statusLabels: Record<GameStatus, string> = {
  ready: "Translation installed",
  available: "Translation available",
  unowned: "Get the game",
  attention: "Needs attention",
  original: "Original language",
};
export function StatusBadge({ status }: { status: GameStatus }) {
  const { t } = useTranslation();
  return (
    <span className={`kt-status kt-status--${status}`}>
      <span aria-hidden="true" />
      {t(statusLabels[status])}
    </span>
  );
}
