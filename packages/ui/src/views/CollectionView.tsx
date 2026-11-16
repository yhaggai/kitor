import { Search } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import { categoryLabel, orderedCategories } from "../categories";
import { Button } from "../components/Button";
import { GameTile } from "../components/GameTile";
import { useTranslation } from "../i18n";
import type { Collection, GameSummary } from "../types";
import { CategoryRow } from "./CategoryRow";

export function CollectionView({
  collection,
  navigationResetKey,
  games,
  loading = false,
  error,
  onRetry,
  onOpen,
  toolbar,
  feedback,
}: {
  collection: Collection;
  navigationResetKey?: number;
  games: GameSummary[];
  loading?: boolean;
  error?: string;
  onRetry?: () => void;
  onOpen: (game: GameSummary, element: HTMLButtonElement) => void;
  toolbar?: ReactNode;
  feedback?: ReactNode;
}) {
  const { t } = useTranslation();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("all");
  // biome-ignore lint/correctness/useExhaustiveDependencies: Navigation explicitly resets the search.
  useEffect(() => {
    setQuery("");
  }, [navigationResetKey]);
  const filters = [
    { id: "all", label: t("All games") },
    { id: "ready", label: t("Translation installed") },
    { id: "available", label: t("Translation available") },
  ];
  const inStatus = (game: GameSummary, status: string) =>
    status === "all" || game.status === status;
  const haystack = (game: GameSummary) =>
    `${game.title} ${game.author ?? ""} ${game.language ?? ""} ${game.searchTerms ?? ""} ${(game.tags ?? []).map((tag) => `${tag} ${t(categoryLabel(tag))}`).join(" ")}`.toLowerCase();
  const matches = games.filter(
    (game) => inStatus(game, filter) && haystack(game).includes(query.trim().toLowerCase())
  );
  const categories = collection === "discovery" ? orderedCategories(matches) : [];
  const uncategorized = categories.length ? matches.filter((game) => !game.tags?.length) : matches;

  return (
    <section
      className="kt-collection"
      aria-label={collection === "library" ? t("My Games") : t("Home")}
    >
      <header className="kt-page-header">
        <div>
          <p className="kt-eyebrow">{t("YOUR NEXT ADVENTURE")}</p>
          <h1>{collection === "library" ? t("My Games") : t("Home")}</h1>
          <p>
            {collection === "library"
              ? t("A little adventure awaits.")
              : t("Familiar worlds. A whole new language.")}
          </p>
        </div>
        {toolbar}
      </header>
      <div className="kt-toolbar">
        <label className="kt-search">
          <Search size={19} />
          <span className="kt-sr-only">{t("Search games")}</span>
          <input
            type="search"
            placeholder={t("Search your next adventure…")}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") setQuery("");
            }}
          />
        </label>
        <fieldset className="kt-filters" aria-label={t("Filter games")}>
          {filters.map(({ id, label }) => (
            <button
              type="button"
              key={id}
              aria-pressed={filter === id}
              onClick={() => setFilter(id)}
            >
              {label}
              <span>{games.filter((game) => inStatus(game, id)).length}</span>
            </button>
          ))}
        </fieldset>
      </div>
      {feedback}
      {loading ? (
        <output className="kt-empty" style={{ display: "block" }}>
          {t("Loading your adventures…")}{" "}
        </output>
      ) : error ? (
        <div className="kt-empty" role="alert">
          <h2>{t("Couldn’t load games")}</h2>
          <p>{error}</p>
          {onRetry && <Button onClick={onRetry}>{t("Try again")}</Button>}
        </div>
      ) : matches.length ? (
        <>
          <output className="kt-sr-only">
            {matches.length} {t("games")}
          </output>
          {collection === "discovery" ? (
            <div className="kt-category-rows" key={navigationResetKey}>
              {categories.map((id) => (
                <CategoryRow
                  key={id}
                  category={id}
                  title={t(categoryLabel(id))}
                  games={matches.filter((game) => game.tags?.includes(id))}
                  onOpen={onOpen}
                />
              ))}
              {uncategorized.length > 0 && (
                <CategoryRow
                  title={t(categories.length ? "Other games" : "All games")}
                  games={uncategorized}
                  onOpen={onOpen}
                />
              )}
            </div>
          ) : (
            <div className="kt-game-grid">
              {matches.map((game) => (
                <GameTile key={game.id} game={game} onOpen={onOpen} showStatus />
              ))}
            </div>
          )}
        </>
      ) : (
        <div className="kt-empty">
          <h2>{games.length ? t("No matching adventures") : t("Your collection starts here")}</h2>
          <p>
            {games.length
              ? t("Try a different title or filter.")
              : t("Add a game to start your next adventure.")}
          </p>
          {games.length > 0 && (
            <Button
              variant="secondary"
              onClick={() => {
                setQuery("");
                setFilter("all");
              }}
            >
              {t("Clear filters")}{" "}
            </Button>
          )}
        </div>
      )}
    </section>
  );
}
