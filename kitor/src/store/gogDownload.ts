import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { type Game, useAppStore } from "./store";

type InstallProgress = { game_id: string; step: string; percentage: number | null };
export const useGogDownload = create<{
  busy: boolean;
  gameId: string | null;
  error: string | null;
  complete: boolean;
  step: string;
  percentage: number | null;
  start: (gogId: number, gameId: string, installRoot: string) => Promise<void>;
}>((set, get) => ({
  busy: false,
  gameId: null,
  error: null,
  complete: false,
  step: "",
  percentage: null,
  start: async (gogId, gameId, installRoot) => {
    if (get().busy) return;
    const account = useAppStore.getState();
    if (!account.gogAuth.isAuthenticated || !account.gogGames.some((game) => game.id === gogId)) {
      set({
        gameId,
        error: "Connect the GOG account that owns this game, then retry.",
        complete: false,
      });
      return;
    }
    set({
      busy: true,
      gameId,
      error: null,
      complete: false,
      step: "Starting installation…",
      percentage: null,
    });
    let unlisten: (() => void) | undefined;
    try {
      unlisten = await listen<InstallProgress>("gog-game-install-progress", ({ payload }) => {
        if (payload.game_id === gameId) set({ step: payload.step, percentage: payload.percentage });
      });
      const game = await invoke<Game>("install_gog_game", {
        gameId: gogId,
        scummvmId: gameId,
        installRoot,
      });
      const store = useAppStore.getState();
      store.setGames(await invoke<Game[]>("get_games_with_patches"));
      store.setHighlightedGameId(game.id);
      set({ complete: true, step: "Ready to play", percentage: 100 });
    } catch (error) {
      set({ error: String(error) });
    } finally {
      unlisten?.();
      set({ busy: false });
    }
  },
}));
