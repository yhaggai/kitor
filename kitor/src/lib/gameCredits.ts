import type { Studio } from "@kitor/ui";
import credits from "@/data/game-credits.json";

const studioCatalog: Record<string, { name: string; logo?: string; logoOnDark?: boolean }> =
  credits.studios;
const studios = new Map<string, Studio>(
  Object.entries(studioCatalog).map(([id, { name, logo, logoOnDark }]) => [
    id,
    { id, name, logo: logo ? `/studio-logos/${logo}` : undefined, logoOnDark },
  ])
);
const games = new Map(Object.entries(credits.games));

export function gameCredits(id: string) {
  const game = games.get(id);
  if (!game) return {};

  return {
    originalDevelopers: game.originalDevelopers.flatMap(
      (developer) => studios.get(developer) ?? []
    ),
    publishers: game.publishers,
  };
}
