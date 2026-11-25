// Refreshed from the app registry and real assets by npm run sync.
import catalog from "../generated/catalog.json";
import type { GameBoxProps } from "../generated/GameBox";
export type CatalogGame = {
  id: string;
  added_on?: string;
  title: string;
  title_he?: string;
  covers: GameBoxProps["covers"];
  platforms: Record<string, boolean>;
  hebrew: boolean;
  descriptionHe?: string;
  descriptionEn?: string;
  sourceUrl?: string;
  screenshot?: string;
  translationCredit?: string;
  translationVersion?: string;
  translationBuilds?: Array<{
    store: string;
    platform: string;
    buildId?: string;
    manifestId?: string;
  }>;
  steamAppId?: number;
  gogSlug?: string;
  gogProductId?: number;
};
export const games: CatalogGame[] = catalog;
export const featured = ["monkey-se", "sq1", "toonstruck", "undertale", "kathy-rain-dc"].map(
  (id) => games.find((g) => g.id === id)!
);

export function gameGuideTitle(game: CatalogGame, locale: "he" | "en") {
  if (locale === "en") return game.hebrew ? `${game.title} in Hebrew – Translation and installation` : game.title;
  const name = game.title_he || game.title;
  const edition = game.id.endsWith("-native") || game.id === "day-of-the-tentacle-remastered"
    ? " (המהדורה המחודשת)"
    : "";
  return game.hebrew ? `${name}${edition} בעברית – תרגום והתקנה` : name;
}
