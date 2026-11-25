import { Tag } from "@kitor/registry";

/** Exhaustive labels: adding a registry tag requires its UI label here. */
export const categoryLabels: Record<Tag, string> = {
  [Tag.HotKitor]: "Hot Kitor",
  [Tag.Classic]: "Old classic",
  [Tag.Detective]: "Detective",
  [Tag.Thriller]: "Thriller",
  [Tag.Indie]: "Indie",
  [Tag.Comedy]: "Comedy",
  [Tag.SciFi]: "Sci-fi",
  [Tag.Fantasy]: "Fantasy",
  [Tag.Kids]: "Kids",
  [Tag.Puzzle]: "Puzzle",
  [Tag.Story]: "Story",
  [Tag.Horror]: "Horror",
};
export type GameCategory = Tag;
export type CategoryFilter = Tag | "all";

export function categoryLabel(tag: Tag): string {
  return categoryLabels[tag];
}

export function orderedCategories(games: readonly { tags?: readonly Tag[] }[]): Tag[] {
  const present = new Set(games.flatMap((game) => game.tags ?? []));
  // Labels define display order; the enum defines the allowed vocabulary.
  return Object.values(Tag)
    .filter((tag) => present.has(tag))
    .sort(
      (a, b) => Object.keys(categoryLabels).indexOf(a) - Object.keys(categoryLabels).indexOf(b)
    );
}

export function hasCategory(tags: readonly Tag[] | undefined, category: CategoryFilter): boolean {
  return category === "all" || (tags ?? []).includes(category);
}
