import { afterEach, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import GameDetails from "../src/components/GameDetails";
import { DialogContext } from "../src/contexts/DialogContext";
import { useSteamStore } from "../src/store/steam";
import { type Game, type PatchWithStatus, useAppStore } from "../src/store/store";

const appInitial = useAppStore.getInitialState();
const initialGames = appInitial.games;
const steamInitial = useSteamStore.getInitialState();
const initialSteam = steamInitial.status;
afterEach(() => {
  appInitial.games = initialGames;
  appInitial.gogAuth = { isAuthenticated: false };
  appInitial.gogGames = [];
  steamInitial.status = initialSteam;
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
  steam_app_id: 319870,
} as PatchWithStatus;
function render(collection: "library" | "discovery", installed?: Game) {
  appInitial.games = installed ? [installed] : [];
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
  expect(discovery).toContain("Install Hebrew translation");
  expect(discovery).toContain("Play without translation");
  expect(discovery).not.toContain("Play!");
  expect(discovery).not.toContain("Purchase Guide");
});

test("discovery actions follow current library state despite stale catalog flags", () => {
  const uninstalled = render("discovery");
  expect(uninstalled).toContain("Get the game");
  expect(uninstalled).toContain("I already have the game");
  expect(uninstalled).not.toContain(">Install<");
  expect(uninstalled).not.toContain("Play!");
  steamInitial.status = {
    steam_id: "1",
    has_api_key: true,
    connecting: false,
    owned_app_ids: [319870],
    message: null,
  };
  const owned = render("discovery");
  expect(owned).toContain(">Install<");
  expect(owned).not.toContain("Get the game");
  expect(owned).not.toContain("I already have the game");
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
  expect(patched).toContain("Play!");
  expect(patched).toContain("With Hebrew translation");
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
  expect(missing).not.toContain("Play!");
  expect(missing).not.toContain("Play without translation");
  expect(missing).toContain("Game files missing");
});
