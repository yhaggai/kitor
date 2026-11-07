import { expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { GameBanner } from "../../packages/ui/src/components/GameBanner";
import registry from "../patches/registry.json";
import banners from "../src/data/game-banners.json";

test("every patch entry has its own published Hebrew banner", () => {
  const urls = new Set<string>();
  for (const game of registry.games.filter((entry) => entry.patches.length)) {
    const banner = banners[game.scummvm_id as keyof typeof banners];
    expect(banner).toBeDefined();
    expect(banner.he).toStartWith(`https://assets.kitor.click/banners/${game.scummvm_id}/he/`);
    expect(banner.aspectRatio).toBe(3);
    expect(banner).not.toHaveProperty("en");
    urls.add(banner.he);
  }
  expect(urls.size).toBe(34);
});

test("Hebrew-only banners render without an English selector or logo", () => {
  const html = renderToStaticMarkup(
    <GameBanner
      game={{ id: "undertale", title: "Undertale", status: "available", banner: banners.undertale }}
    />
  );
  expect(html).toContain(banners.undertale.he);
  expect(html).toContain("Hebrew banner");
  expect(html).not.toContain("English");
  expect(html).not.toContain("fieldset");
  expect(html).not.toContain("kt-game-banner-logo");
});
