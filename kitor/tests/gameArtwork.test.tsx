import { expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { GameCover } from "../../packages/ui/src/components/GameCover";
import { getCatalogCovers, getCoverSrc } from "../src/lib/utils";

test("retired desktop artwork falls back to the game title", () => {
  const cover = getCoverSrc("/monkey-classic-en.jpg");
  const covers = getCatalogCovers("monkey-se");
  expect(cover).toBeNull();
  expect(covers?.en.src).toBe("");
  expect(covers?.he.src).toBe("");

  const html = renderToStaticMarkup(
    <GameCover game={{ id: "monkey-se", title: "Monkey Island", status: "available", covers }} />
  );
  expect(html).toContain("Monkey Island");
  expect(html).not.toContain("<img");
});

test("a remote cover remains available when the other language has no local artwork", () => {
  const covers = getCatalogCovers("drifter");
  expect(covers?.en.src).toBe("");
  expect(covers?.he.src).toStartWith("https://assets.kitor.click/");
});
