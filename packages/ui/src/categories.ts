/** Stable registry tag ids and the English labels shown on Home. */
export const categoryLabels = {
  classic: "Old classic",
  detective: "Detective",
  thriller: "Thriller",
  indie: "Indie",
  comedy: "Comedy",
  "sci-fi": "Sci-fi",
  fantasy: "Fantasy",
  kids: "Kids",
  puzzle: "Puzzle",
  story: "Story",
  horror: "Horror",
} as const;

export type GameCategory = keyof typeof categoryLabels;

export function categoryLabel(tag: string): string {
  return tag in categoryLabels ? categoryLabels[tag as GameCategory] : tag;
}

/** Known categories first, then any registry tag that does not have a label yet. */
export function orderedCategories(games: readonly { tags?: readonly string[] }[]): string[] {
  const present = new Set(games.flatMap((game) => game.tags ?? []));
  const known = (Object.keys(categoryLabels) as string[]).filter((id) => present.has(id));
  const extra = [...present].filter((id) => !(id in categoryLabels)).sort();
  return [...known, ...extra];
}

export function hasCategory(tags: readonly string[] | undefined, category: string): boolean {
  return category === "all" || (tags ?? []).includes(category);
}
