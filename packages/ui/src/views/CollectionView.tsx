import * as Dropdown from "@radix-ui/react-dropdown-menu";
import { Check, ListFilter, Search, SlidersHorizontal, X } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import { categoryLabel, orderedCategories } from "../categories";
import { Button } from "../components/Button";
import { type GameSortOrder, sortGamesByRelease } from "../components/GameSort";
import { GameTile } from "../components/GameTile";
import { useTranslation } from "../i18n";
import type { Collection, GameSummary, Studio } from "../types";
import { CategoryRow } from "./CategoryRow";

const HOME_VIEW_STORAGE_KEY = "kitor-home-view";

type HomeView = "categories" | "grid" | "studios" | "years";

function MenuOption({
  value,
  disabled,
  children,
}: {
  value: string;
  disabled?: boolean;
  children: ReactNode;
}) {
  return (
    <Dropdown.RadioItem className="kt-collection-option" value={value} disabled={disabled}>
      <Dropdown.ItemIndicator>
        <Check size={15} aria-hidden="true" />
      </Dropdown.ItemIndicator>
      <span>{children}</span>
    </Dropdown.RadioItem>
  );
}

function readHomeView(): HomeView {
  try {
    const view = globalThis.localStorage?.getItem(HOME_VIEW_STORAGE_KEY);
    return view === "grid" || view === "studios" || view === "years" ? view : "categories";
  } catch {
    return "categories";
  }
}

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
  const { t, language } = useTranslation();
  const direction = language === "he" ? "rtl" : "ltr";
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("all");
  const [homeSort, setHomeSort] = useState<GameSortOrder>("default");
  const [categorySorts, setCategorySorts] = useState<Record<string, GameSortOrder>>({});
  const isHome = collection === "discovery";
  const [homeView, setHomeView] = useState(readHomeView);
  const searchTerm = query.trim().toLowerCase();
  const isBrowsingHome = isHome && !searchTerm;
  const showCategories = isBrowsingHome && homeView === "categories";
  const showStudios = isBrowsingHome && homeView === "studios";
  const showYears = isBrowsingHome && homeView === "years";

  function changeHomeView(view: HomeView) {
    setHomeView(view);
    try {
      globalThis.localStorage?.setItem(HOME_VIEW_STORAGE_KEY, view);
    } catch {
      // Switching views still works when persistent storage is unavailable.
    }
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: Navigation explicitly resets the search.
  useEffect(() => {
    setQuery("");
  }, [navigationResetKey]);
  const filters = [
    { id: "all", label: t("All games") },
    { id: "ready", label: t("Translation installed") },
    { id: "available", label: t("Translation available") },
  ];
  const activeFilter = filters.find(({ id }) => id !== "all" && id === filter);
  const inStatus = (game: GameSummary, status: string) =>
    status === "all" || game.status === status;
  const haystack = (game: GameSummary) =>
    `${game.title} ${game.author ?? ""} ${game.language ?? ""} ${game.searchTerms ?? ""} ${(game.originalDevelopers ?? []).map((studio) => studio.name).join(" ")} ${(game.tags ?? []).map((tag) => `${tag} ${t(categoryLabel(tag))}`).join(" ")}`.toLowerCase();
  const matches = games.filter(
    (game) => inStatus(game, filter) && haystack(game).includes(searchTerm)
  );
  const categories = showCategories ? orderedCategories(matches) : [];
  const studios = new Map<string, { studio: Studio; games: GameSummary[] }>();
  if (showStudios) {
    for (const game of matches) {
      const developers = new Map(game.originalDevelopers?.map((studio) => [studio.id, studio]));
      for (const studio of developers.values()) {
        const row = studios.get(studio.id);
        if (row) row.games.push(game);
        else studios.set(studio.id, { studio, games: [game] });
      }
    }
  }
  const studioRows = [...studios.values()].sort((a, b) =>
    a.studio.name.localeCompare(b.studio.name)
  );
  const years = new Map<string, GameSummary[]>();
  const undated: GameSummary[] = [];
  if (showYears) {
    for (const game of matches) {
      const year = game.originalReleaseDate?.slice(0, 4);
      if (!year) {
        undated.push(game);
        continue;
      }
      const row = years.get(year);
      if (row) row.push(game);
      else years.set(year, [game]);
    }
  }
  const yearRows = [...years.entries()].sort(([a], [b]) =>
    homeSort === "release-oldest" ? a.localeCompare(b) : b.localeCompare(a)
  );
  const ungrouped = matches.filter((game) =>
    showStudios ? !game.originalDevelopers?.length : !game.tags?.length
  );

  function categorySortProps(id: string) {
    const sortOrder = categorySorts[id] ?? homeSort;
    return {
      sortOrder,
      onSortChange: (order: GameSortOrder) =>
        setCategorySorts((previous) => ({ ...previous, [id]: order })),
    };
  }

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
      <div className="kt-collection-controls">
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
          {isHome ? (
            <div className="kt-collection-menu-actions">
              <Dropdown.Root dir={direction} modal={false}>
                <Dropdown.Trigger
                  className="kt-collection-menu-trigger"
                  aria-label={t("Filter games")}
                  title={t("Filter games")}
                  data-filtered={filter !== "all" || undefined}
                >
                  <ListFilter size={17} aria-hidden="true" />
                </Dropdown.Trigger>
                <Dropdown.Content className="kt-collection-menu" align="end" sideOffset={6}>
                  <Dropdown.RadioGroup
                    aria-label={t("Translation status")}
                    value={filter}
                    onValueChange={setFilter}
                  >
                    <Dropdown.Label className="kt-collection-menu-label">
                      {t("Translation status")}
                    </Dropdown.Label>
                    {filters.map(({ id, label }) => (
                      <MenuOption key={id} value={id}>
                        {label} ({games.filter((game) => inStatus(game, id)).length})
                      </MenuOption>
                    ))}
                  </Dropdown.RadioGroup>
                </Dropdown.Content>
              </Dropdown.Root>
              <Dropdown.Root dir={direction} modal={false}>
                <Dropdown.Trigger
                  className="kt-collection-menu-trigger"
                  aria-label={t("Display options")}
                  title={t("Display options")}
                >
                  <SlidersHorizontal size={17} aria-hidden="true" />
                </Dropdown.Trigger>
                <Dropdown.Content className="kt-collection-menu" align="end" sideOffset={6}>
                  <Dropdown.RadioGroup
                    aria-label={t("Grouping")}
                    value={isBrowsingHome ? homeView : "grid"}
                    onValueChange={(value) => changeHomeView(value as HomeView)}
                  >
                    <Dropdown.Label className="kt-collection-menu-label">
                      {t("Grouping")}
                    </Dropdown.Label>
                    <MenuOption value="categories" disabled={!isBrowsingHome}>
                      {t("Categories")}
                    </MenuOption>
                    <MenuOption value="studios" disabled={!isBrowsingHome}>
                      {t("Studios")}
                    </MenuOption>
                    <MenuOption value="years" disabled={!isBrowsingHome}>
                      {t("Release year")}
                    </MenuOption>
                    <MenuOption value="grid" disabled={!isBrowsingHome}>
                      {t("No grouping")}
                    </MenuOption>
                  </Dropdown.RadioGroup>
                  <Dropdown.Separator className="kt-collection-menu-separator" />
                  <Dropdown.RadioGroup
                    aria-label={t("Ordering")}
                    value={homeSort}
                    onValueChange={(value) => {
                      setHomeSort(value as GameSortOrder);
                      if (isBrowsingHome) setCategorySorts({});
                    }}
                  >
                    <Dropdown.Label className="kt-collection-menu-label">
                      {t("Ordering")}
                    </Dropdown.Label>
                    <MenuOption value="default">{t("Default order")}</MenuOption>
                    <MenuOption value="release-newest">
                      {t("Release date (newest first)")}
                    </MenuOption>
                    <MenuOption value="release-oldest">
                      {t("Release date (oldest first)")}
                    </MenuOption>
                  </Dropdown.RadioGroup>
                </Dropdown.Content>
              </Dropdown.Root>
            </div>
          ) : (
            <select
              className="kt-game-filter"
              aria-label={t("Filter games")}
              value={filter}
              onChange={(event) => setFilter(event.target.value)}
            >
              {filters.map(({ id, label }) => (
                <option key={id} value={id}>
                  {label} ({games.filter((game) => inStatus(game, id)).length})
                </option>
              ))}
            </select>
          )}
        </div>
        {isHome && activeFilter && (
          <button
            className="kt-collection-filter-chip"
            type="button"
            aria-label={t("Remove filter: {0}", [activeFilter.label])}
            onClick={() => setFilter("all")}
          >
            {activeFilter.label}
            <X size={14} aria-hidden="true" />
          </button>
        )}
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
          {showCategories || showStudios || showYears ? (
            <div className="kt-category-rows" key={navigationResetKey}>
              {categories.map((id) => (
                <CategoryRow
                  key={id}
                  category={id}
                  title={t(categoryLabel(id))}
                  games={matches.filter((game) => game.tags?.includes(id))}
                  onOpen={onOpen}
                  {...categorySortProps(id)}
                />
              ))}
              {studioRows.map(({ studio, games: studioGames }) => (
                <CategoryRow
                  key={studio.id}
                  title={studio.name}
                  logo={studio.logo}
                  logoOnDark={studio.logoOnDark}
                  games={studioGames}
                  onOpen={onOpen}
                  {...categorySortProps(`studio:${studio.id}`)}
                />
              ))}
              {yearRows.map(([year, yearGames]) => (
                <CategoryRow
                  key={year}
                  title={year}
                  games={yearGames}
                  onOpen={onOpen}
                  {...categorySortProps(`year:${year}`)}
                />
              ))}
              {showYears && undated.length > 0 && (
                <CategoryRow
                  title={t("Unknown release year")}
                  games={undated}
                  onOpen={onOpen}
                  {...categorySortProps("year:unknown")}
                />
              )}
              {!showYears && ungrouped.length > 0 && (
                <CategoryRow
                  title={t(
                    showStudios
                      ? "Uncredited developers"
                      : categories.length
                        ? "Other games"
                        : "All games"
                  )}
                  games={ungrouped}
                  onOpen={onOpen}
                  {...categorySortProps(showStudios ? "studio:uncredited" : "uncategorized")}
                />
              )}
            </div>
          ) : (
            <div className="kt-game-grid">
              {sortGamesByRelease(matches, isHome ? homeSort : "default").map((game) => (
                <GameTile
                  key={game.id}
                  game={game}
                  onOpen={onOpen}
                  showStatus={!isHome}
                  showBeta={isHome && game.translationBeta}
                />
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
