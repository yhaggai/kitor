import { afterEach, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { categoryLabels, hasCategory, orderedCategories } from "../../packages/ui/src/categories";
import { setAppLanguage } from "../../packages/ui/src/i18n";
import type { GameSummary } from "../../packages/ui/src/types";
import { CollectionView } from "../../packages/ui/src/views/CollectionView";
import { patchSummary } from "../src/lib/presentation";
import type { PatchWithStatus } from "../src/store/store";

afterEach(() => setAppLanguage("en"));

const catalog: GameSummary[] = [
  {
    id: "monkey",
    title: "The Secret of Monkey Island",
    status: "unowned",
    tags: ["classic", "comedy"],
  },
  {
    id: "kathy",
    title: "Kathy Rain",
    status: "available",
    tags: ["indie", "detective", "thriller"],
  },
];

test("every patched game has one or more known category tags", async () => {
  const { default: registry } = await import("../patches/registry.json");
  const known = new Set(Object.keys(categoryLabels));
  const tagged = registry.games.filter((game) => game.patches.length > 0);
  expect(tagged.length).toBeGreaterThan(0);
  for (const game of tagged) {
    const tags = game.tags ?? [];
    expect(tags.length).toBeGreaterThan(0);
    expect(new Set(tags).size).toBe(tags.length);
    for (const tag of tags) expect(known.has(tag)).toBe(true);
  }
  for (const game of registry.games.filter((entry) => entry.patches.length === 0)) {
    expect(game.tags).toBeUndefined();
  }
  expect(tagged.some((game) => game.tags.length > 1)).toBe(true);
});

test("discovery cards keep every registry tag", () => {
  const patch = {
    scummvm_id: "kathy-rain-dc",
    game_title: "Kathy Rain",
    language: "he",
    version: "1",
    tags: ["indie", "detective", "thriller"],
    game_installed: false,
    patch_installed: false,
  } as PatchWithStatus;
  expect(patchSummary(patch).tags).toEqual(["indie", "detective", "thriller"]);
  expect(patchSummary(patch).searchTerms).toContain("detective");
  expect(hasCategory(patch.tags, "thriller")).toBe(true);
  expect(hasCategory(patch.tags, "classic")).toBe(false);
  expect(orderedCategories(catalog)).toEqual([
    "classic",
    "detective",
    "thriller",
    "indie",
    "comedy",
  ]);
});

test("home lists category filters and the library does not", () => {
  const home = renderToStaticMarkup(
    <CollectionView collection="discovery" games={catalog} onOpen={() => {}} />
  );
  expect(home).toContain("Categories");
  expect(home).toContain("Old classic");
  expect(home).toContain("Detective");
  expect(home).toContain("Thriller");
  const library = renderToStaticMarkup(
    <CollectionView collection="library" games={catalog} onOpen={() => {}} />
  );
  expect(library).not.toContain("Old classic");
  expect(library).toContain("All games");
  setAppLanguage("he");
  const hebrew = renderToStaticMarkup(
    <CollectionView collection="discovery" games={catalog} onOpen={() => {}} />
  );
  expect(hebrew).toContain("קלאסיקות");
  expect(hebrew).toContain("בלשים");
  expect(hebrew).toContain("מתח");
});
