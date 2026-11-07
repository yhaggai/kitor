import { expect, test } from "bun:test";
import { gameSummary, patchSummary } from "../src/lib/presentation";
import type { Game, PatchWithStatus } from "../src/store/store";

const game: Game = {
  id: 7,
  scummvm_id: "undertale",
  title: "UNDERTALE",
  install_path: "/test",
  added_date: "2026-01-01",
  has_available_patch: true,
  is_patched: true,
};

test("disk health overrides an old installed flag in the redesigned library", () => {
  expect(gameSummary(game).status).toBe("ready");
  expect(
    gameSummary({
      ...game,
      health: {
        game_status: "present",
        patch_status: "damaged",
        details: [],
        checked_at: "2026-01-01",
      },
    }).status
  ).toBe("attention");
  expect(
    gameSummary({
      ...game,
      health: {
        game_status: "missing",
        patch_status: "installed",
        details: [],
        checked_at: "2026-01-01",
      },
    }).status
  ).toBe("attention");
  expect(
    gameSummary({
      ...game,
      health: {
        game_status: "present",
        patch_status: "not_installed",
        details: [],
        checked_at: "2026-01-01",
      },
    }).status
  ).toBe("available");
});

test("translation variants keep distinct navigation identities", () => {
  const patch = {
    scummvm_id: "undertale",
    game_title: "UNDERTALE",
    language: "he",
    version: "1",
    game_installed: false,
    patch_installed: false,
  } as PatchWithStatus;
  expect(patchSummary(patch).id).not.toBe(patchSummary({ ...patch, language: "he-nikud" }).id);
  expect(patchSummary(patch).status).toBe("unowned");
  expect(patchSummary({ ...patch, game_installed: true }).status).toBe("available");
});

test("every registered game has a Hebrew editorial synopsis without setup instructions", async () => {
  const { default: registry } = await import("../patches/registry.json");
  const { default: descriptions } = await import("../src/data/game-descriptions.he.json");
  for (const entry of registry.games) {
    const description = descriptions[entry.scummvm_id as keyof typeof descriptions];
    expect(description).toBeDefined();
    expect(description.description).toMatch(/[א-ת]/);
    expect(description.description).not.toMatch(
      /https?:|הוראות התקנה|הוראות הפעלה|הורידו|התקינו|Python|ScummVM|remonster/i
    );
  }
});

test("library and discovery never use technical patch notes as the game description", async () => {
  const { gameDescription, translationCommunity } = await import("../src/lib/gameDescriptions");
  const patch = {
    scummvm_id: "portal2",
    game_title: "Portal 2",
    language: "he",
    version: "1",
    description: "Install Python and copy files",
    game_installed: true,
    patch_installed: false,
  } as PatchWithStatus;
  expect(patchSummary(patch).description).toBe(gameDescription("portal2").description);
  expect(gameSummary({ ...game, scummvm_id: "portal2" }).description).toBe(
    patchSummary(patch).description
  );
  expect(gameDescription("portal2").communityTranslation).toBe(true);
  expect(translationCommunity.url).toBe("https://www.facebook.com/groups/hebrewadventure");
  expect(gameDescription("jenny-leclue").communityTranslation).toBe(false);
});
