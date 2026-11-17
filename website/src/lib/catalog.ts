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
  steamAppId?: number;
  gogSlug?: string;
  gogProductId?: number;
};
export const games: CatalogGame[] = catalog;
export const featured = ["monkey-se", "sq1", "toonstruck", "undertale", "kathy-rain-dc"].map(
  (id) => games.find((g) => g.id === id)!
);
