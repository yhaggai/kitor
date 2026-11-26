import { type GameSummary, useTranslation } from "@kitor/ui";
import { Button } from "@/components/ui/button";
import { relatedGames } from "@/lib/relatedGames";

type Entry = { gameId: string; summary: GameSummary };

export default function RelatedGames({
  currentId,
  entries,
  onSelect,
}: {
  currentId?: string;
  entries: Entry[];
  onSelect: (id: string) => void;
}) {
  const { language } = useTranslation();
  const catalog = Array.from(
    new Map(
      entries.map(
        ({ gameId, summary }) =>
          [gameId, { ...summary, id: gameId, navigationId: summary.id }] as const
      )
    ).values()
  );
  const current = catalog.find((game) => game.id === currentId);
  const suggestions = current ? relatedGames(current, catalog) : [];
  if (!suggestions.length) return null;
  return (
    <section className="mt-6 space-y-3">
      <h2 className="text-xl font-semibold">
        {language === "he" ? "משחקים נוספים שאולי יעניינו אתכם" : "Games you might also like"}
      </h2>
      <div className="flex flex-wrap gap-2">
        {suggestions.map((game) => (
          <Button key={game.id} variant="outline" onClick={() => onSelect(game.navigationId)}>
            <bdi>{language === "he" ? game.titleHe || game.title : game.title}</bdi>
          </Button>
        ))}
      </div>
    </section>
  );
}
