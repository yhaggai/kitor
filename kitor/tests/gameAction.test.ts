import { expect, test } from "bun:test";
import registry from "../patches/registry.json";
import { gogStoreUrl, ownsGame, primaryGameAction } from "../src/lib/gameAction";

test("the primary action follows purchase, install, and patch state", () => {
  expect(
    primaryGameAction({
      inLibrary: false,
      filesReady: false,
      patched: false,
      patchAvailable: true,
      purchased: false,
    })
  ).toBe("buy");
  expect(
    primaryGameAction({
      inLibrary: false,
      filesReady: false,
      patched: false,
      patchAvailable: true,
      purchased: true,
    })
  ).toBe("install");
  expect(
    primaryGameAction({
      inLibrary: true,
      filesReady: true,
      patched: false,
      patchAvailable: true,
      purchased: true,
    })
  ).toBe("install-patch");
  expect(
    primaryGameAction({
      inLibrary: true,
      filesReady: true,
      patched: true,
      patchAvailable: true,
      purchased: true,
    })
  ).toBe("play");
  expect(
    primaryGameAction({
      inLibrary: true,
      filesReady: false,
      patched: false,
      patchAvailable: true,
      purchased: true,
    })
  ).toBe("unavailable");
});

test("ownership is recognized from either store, and GOG store links use the installer slug", () => {
  expect(ownsGame({ steamAppId: 10, gogId: 20, ownedSteamAppIds: [10], ownedGogIds: [] })).toBe(
    true
  );
  expect(ownsGame({ steamAppId: 10, gogId: 20, ownedSteamAppIds: [], ownedGogIds: [20] })).toBe(
    true
  );
  expect(ownsGame({ steamAppId: 10, gogId: 20, ownedSteamAppIds: null, ownedGogIds: [] })).toBe(
    false
  );
  expect(
    gogStoreUrl(
      "https://www.gog.com/downloads/the_secret_of_monkey_island_special_edition/en1installer0"
    )
  ).toBe("https://www.gog.com/en/game/the_secret_of_monkey_island_special_edition");
  expect(gogStoreUrl(null)).toBeNull();
  expect(gogStoreUrl("https://www.gog.com/downloads/the_dig/en1installer0")).toBe(
    "https://www.gog.com/en/game/the_dig"
  );
});

const withoutGogListing = new Set([
  "spyfox",
  "stanley-parable",
  "portal2",
  "silver-pines",
  "tentacle",
]);

test("each registered game stores the GOG add-to-cart link for the product that sells it", () => {
  for (const game of registry.games) {
    const url = "add_to_cart_url" in game ? game.add_to_cart_url : undefined;
    if (withoutGogListing.has(game.scummvm_id)) {
      expect(url).toBeUndefined();
      continue;
    }
    expect(url).toMatch(
      /^https:\/\/www\.gog\.com\/checkout\/widget\/52756712356612660\/\d+\?price=/
    );
  }
  const dig = registry.games.find((game) => game.scummvm_id === "dig");
  expect(dig && "add_to_cart_url" in dig ? dig.add_to_cart_url : undefined).toBe(
    "https://www.gog.com/checkout/widget/52756712356612660/1425299904?price=undefined%2018.60"
  );
});
