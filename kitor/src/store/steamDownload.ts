import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import { type Game, useAppStore } from "./store";

export const useSteamDownload = create<{
  busy: boolean;
  appId: number | null;
  error: string | null;
  start: (appId: number, gameIds: string[]) => Promise<void>;
}>((set, get) => ({
  busy: false,
  appId: null,
  error: null,
  start: async (appId, gameIds) => {
    if (get().busy) return;
    set({ busy: true, appId, error: null });
    try {
      const games = await invoke<Game[]>("download_steam_windows_game", { appId, gameIds });
      const store = useAppStore.getState();
      store.setGames(await invoke<Game[]>("get_games_with_patches"));
      store.setHighlightedGameId(games.find((game) => game.scummvm_id === gameIds[0])?.id ?? games[0]?.id ?? null);
      store.setActiveTab("library");
    } catch (error) {
      set({ error: String(error) });
    } finally {
      try { useAppStore.getState().setGames(await invoke<Game[]>("get_games_with_patches")); } catch { /* Preserve the original error. */ }
      set({ busy: false });
    }
  },
}));
