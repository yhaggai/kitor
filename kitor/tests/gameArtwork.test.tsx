import { expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { GameCover } from "../../packages/ui/src/components/GameCover";
import { getCatalogCovers, getCoverSrc } from "../src/lib/utils";

test("retired desktop artwork falls back to the game title", () => {
  const cover = getCoverSrc("/drifter-en.jpg");
  const covers = getCatalogCovers("unknown-game");
  expect(cover).toBeNull();
  expect(covers).toBeUndefined();

  const html = renderToStaticMarkup(
    <GameCover game={{ id: "unknown-game", title: "Unknown Game", status: "available", covers }} />
  );
  expect(html).toContain("Unknown Game");
  expect(html).not.toContain("<img");
});

test("a remote cover remains available when the other language has no local artwork", () => {
  const covers = getCatalogCovers("drifter");
  expect(covers?.en.src).toBe("");
  expect(covers?.he.src).toStartWith("https://assets.kitor.click/");
});

test("renders hosted Silver Pines covers that match the English key art", () => {
  const covers = getCatalogCovers("silver-pines");
  const enUrl =
    "https://assets.kitor.click/cdn-cgi/image/width=480,fit=scale-down,quality=85,format=auto/covers/silver-pines/en/d277d9e48633.jpg";
  const heUrl =
    "https://assets.kitor.click/cdn-cgi/image/width=480,fit=scale-down,quality=85,format=auto/covers/silver-pines/he/5a22025b1608.jpg";
  expect(covers?.en.src).toBe(enUrl);
  expect(covers?.he.src).toBe(heUrl);

  const html = renderToStaticMarkup(
    <GameCover game={{ id: "silver-pines", title: "Silver Pines", status: "available", covers }} />
  );
  expect(html).toContain(`src="${enUrl}"`);
  expect(html).toContain(`src="${heUrl}"`);
});

test.each([
  ["monkey-se", "the-secret-of-monkey-island", "f16669f4012f.png"],
  ["monkey2-se", "monkey-island-2-lechuck-s-revenge", "926eac6d9534.jpg"],
  ["tentacle", "tentacle", "b6408a56abfe.png?v=1"],
  ["day-of-the-tentacle-remastered", "day-of-the-tentacle-remastered", "aa8b5a6b1c67.png?v=1"],
  ["grim-fandango-remastered", "grim-fandango-remastered", "a1c08634f446.png"],
])("renders the hosted Hebrew cover for %s", (id, catalogId, filename) => {
  const covers = getCatalogCovers(id);
  const hebrewUrl = `https://assets.kitor.click/covers/${catalogId}/he/${filename}`;
  expect(covers?.he.src).toBe(hebrewUrl);

  const html = renderToStaticMarkup(
    <GameCover game={{ id, title: id, status: "available", covers }} />
  );
  expect(html).toContain(`src="${hebrewUrl}"`);
  expect(html).toContain('data-language="he"');
});
