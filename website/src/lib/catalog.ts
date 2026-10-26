// Refreshed from the app registry and real assets by npm run sync.
import catalog from "../generated/catalog.json";
import type { GameBoxProps } from "../generated/GameBox";
export const games = catalog as Array<{
  id: string;
  title: string;
  covers: GameBoxProps["covers"];
  platforms: Record<string, boolean>;
  hebrew: boolean;
}>;
export const featured = [
  "monkey-se",
  "sq1",
  "toonstruck",
  "undertale",
  "kathy-rain-dc",
].map((id) => games.find((g) => g.id === id)!);
