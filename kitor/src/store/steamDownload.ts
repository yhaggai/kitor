import { invoke } from "@tauri-apps/api/core";
import { ask } from "@tauri-apps/plugin-dialog";
import { create } from "zustand";
import { type Game, useAppStore } from "./store";

export const useSteamDownload = create<{
  busy: boolean;
  appId: number | null;
  error: string | null;
  start: (appId: number, gameIds: string[], installRoot: string) => Promise<void>;
}>((set, get) => ({
  busy: false,
  appId: null,
  error: null,
  start: async (appId, gameIds, installRoot) => {
    if (get().busy) return;
    set({ busy: true, appId, error: null });
    try {
      const download = (overwrite = false) =>
        invoke<Game[]>("download_steam_windows_game", {
          appId,
          gameIds,
          installRoot,
          overwrite,
        });
      let games: Game[];
      try {
        games = await download();
      } catch (error) {
        const message = String(error);
        if (!message.startsWith("OVERWRITE_REQUIRED:")) throw error;
        const path = message.slice("OVERWRITE_REQUIRED:".length);
        const confirmed = await ask(
          `The installation folder already exists:\n${path}\n\nContinuing will replace this folder and delete any saves, patches, or other files inside it.`,
          {
            title: "Replace existing installation?",
            kind: "warning",
            okLabel: "Replace folder",
            cancelLabel: "Cancel",
          }
        );
        if (!confirmed) return;
        games = await download(true);
      }
      const store = useAppStore.getState();
      store.setGames(await invoke<Game[]>("get_games_with_patches"));
      store.setHighlightedGameId(
        games.find((game) => game.scummvm_id === gameIds[0])?.id ?? games[0]?.id ?? null
      );
      store.setActiveTab("library");
    } catch (error) {
      set({ error: String(error) });
    } finally {
      try {
        useAppStore.getState().setGames(await invoke<Game[]>("get_games_with_patches"));
      } catch {
        /* Preserve the original error. */
      }
      set({ busy: false });
    }
  },
}));
