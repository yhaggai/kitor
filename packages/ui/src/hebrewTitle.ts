import type { AppLanguage } from "./i18n";
import type { GameSummary } from "./types";

/** Hebrew when the app language is Hebrew. The English setting keeps the English title. */
export function gameTitle(
  game: Pick<GameSummary, "title" | "titleHe" | "covers">,
  language: AppLanguage
): string {
  if (language === "he")
    return game.titleHe?.trim() || game.covers?.he?.title?.trim() || game.title;
  return game.title;
}
