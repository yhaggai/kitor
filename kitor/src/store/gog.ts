import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import { type GogGame, useAppStore } from "./store";

let generation = 0;
let pending: Promise<void> | null = null;

export const useGogLibrary = create<{
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  clear: () => void;
}>((set) => ({
  loading: false,
  error: null,
  refresh: () => {
    if (pending) return pending;
    const current = ++generation;
    set({ loading: true, error: null });
    pending = (async () => {
      try {
        const isAuthenticated = await invoke<boolean>("check_gog_auth_status");
        if (current !== generation) return;
        useAppStore.getState().setGogAuth({ isAuthenticated });
        if (!isAuthenticated) {
          useAppStore.getState().setGogGames([]);
          return;
        }
        const games = await invoke<GogGame[]>("get_gog_user_games");
        if (current === generation) useAppStore.getState().setGogGames(games);
      } catch (error) {
        if (current === generation) set({ error: String(error) });
      } finally {
        if (current === generation) {
          pending = null;
          set({ loading: false });
        }
      }
    })();
    return pending;
  },
  clear: () => {
    ++generation;
    pending = null;
    useAppStore.getState().setGogAuth({ isAuthenticated: false });
    useAppStore.getState().setGogGames([]);
    set({ loading: false, error: null });
  },
}));
