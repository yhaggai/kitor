import { afterEach, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import GameDetails from "../src/components/GameDetails";
import { DialogContext } from "../src/contexts/DialogContext";
import { type Game, type PatchWithStatus, useAppStore } from "../src/store/store";

const initialGames = useAppStore.getInitialState().games;
afterEach(() => {
  useAppStore.getInitialState().games = initialGames;
});
const game: Game = {
  id: 7,
  scummvm_id: "sq1sci",
  title: "Space Quest",
  install_path: "/games/sq1",
  added_date: "2026-01-01",
  has_available_patch: true,
  available_patches: [
    { language: "he", version: "2", release_revision: 2, author: "Translator", description: "" },
  ],
};
const patch = {
  scummvm_id: "sq1sci",
  game_title: "Space Quest",
  language: "he",
  version: "2",
  game_installed: false,
  patch_installed: false,
} as PatchWithStatus;
function render(collection: "library" | "discovery", installed?: Game) {
  useAppStore.getInitialState().games = installed ? [installed] : [];
  return renderToStaticMarkup(
    <DialogContext.Provider
      value={{
        alert: async () => {},
        confirm: async () => false,
        showDialog: () => {},
        hideDialog: () => {},
      }}
    >
      <GameDetails
        {...(collection === "library" && installed ? { game: installed } : { patch })}
        collection={collection}
        onBack={() => {}}
      />
    </DialogContext.Provider>
  );
}

test("installed games have identical details and actions from either collection", () => {
  const library = render("library", game);
  const discovery = render("discovery", game);
  expect(discovery.replace("Back to discovery", "Back to library")).toBe(library);
  expect(discovery).toContain("Play");
  expect(discovery).toContain("Install Patch");
  expect(discovery).not.toContain("Purchase Guide");
});

test("discovery actions follow current library state despite stale catalog flags", () => {
  const uninstalled = render("discovery");
  expect(uninstalled).not.toContain(">Play<");
  const patched = render("discovery", {
    ...game,
    is_patched: true,
    installed_patches: [
      {
        id: 1,
        scummvm_id: game.scummvm_id,
        language: "he",
        version: "1",
        release_revision: 1,
        status: "active",
        installed_date: "2026-01-01",
      },
    ],
  });
  expect(patched).toContain("Play");
  expect(patched).toContain("Install Patch Update");
  const missing = render("discovery", {
    ...game,
    health: {
      game_status: "missing",
      patch_status: "not_installed",
      details: [],
      checked_at: "2026-01-01",
    },
  });
  expect(missing).not.toContain(">Play<");
  expect(missing).toContain("Game files missing");
});
