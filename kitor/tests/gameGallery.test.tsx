import { expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { renderToStaticMarkup } from "react-dom/server";
import { GameGallery } from "../../packages/ui/src/components/GameGallery";
import galleries from "../src/data/game-galleries.json";
import { gameSummary, patchSummary } from "../src/lib/presentation";

test("all gallery files are bundled locally and retain their Facebook provenance", () => {
  for (const images of Object.values(galleries)) {
    expect(images.length).toBeGreaterThan(0);
    expect(new Set(images.map((image) => image.src)).size).toBe(images.length);
    for (const image of images) {
      expect(image.src).toStartWith("/screenshots/");
      expect(existsSync(resolve(import.meta.dir, "../public", image.src.slice(1)))).toBe(true);
      expect(new URL(image.sourceUrl).hostname).toBe("www.facebook.com");
    }
  }
});

test("games without screenshots do not show an empty gallery", () => {
  expect(renderToStaticMarkup(<GameGallery title="Unknown" />)).toBe("");
});

test("both installed games and discovery patches receive their gallery", () => {
  // These adapters only read the listed fields for a minimal uninstalled game/patch.
  const game = gameSummary({ id: 1, scummvm_id: "sq1sci", title: "Space Quest" } as Parameters<
    typeof gameSummary
  >[0]);
  const patch = patchSummary({
    scummvm_id: "sq1sci",
    game_title: "Space Quest",
    language: "he",
    version: "1",
  } as Parameters<typeof patchSummary>[0]);
  expect(game.screenshots).toEqual(galleries.sq1sci);
  expect(patch.screenshots).toEqual(galleries.sq1sci);
});
