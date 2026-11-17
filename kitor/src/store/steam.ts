import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export type SteamStatus = {
  steam_id: string | null;
  has_api_key: boolean;
  connecting: boolean;
  owned_app_ids: number[] | null;
  message: string | null;
};
export type InitialSteamLibrary = {
  library: { steam_id: string; owned_app_ids: number[] } | null;
  error: string | null;
};
const empty: SteamStatus = {
  steam_id: null,
  has_api_key: false,
  connecting: false,
  owned_app_ids: null,
  message: null,
};
let request = 0;
export const useSteamStore = create<{
  status: SteamStatus;
  busy: boolean;
  error: string | null;
  reset: () => void;
  applySignIn: (result: InitialSteamLibrary) => void;
  run: (command: string, args?: Record<string, unknown>) => Promise<boolean>;
}>((set) => ({
  status: empty,
  busy: false,
  error: null,
  reset: () => {
    request += 1;
    set({ status: { ...empty }, busy: false, error: null });
  },
  applySignIn: ({ library, error }) => {
    request += 1;
    set({ status: { ...empty, ...library }, busy: false, error });
  },
  run: async (command, args) => {
    const current = ++request;
    set({ busy: true, error: null });
    if (command === "connect_steam") set((s) => ({ status: { ...s.status, connecting: true } }));
    try {
      const status = await invoke<SteamStatus>(command, args);
      if (current === request) set({ status });
      return true;
    } catch (error) {
      if (current === request)
        set((s) => ({
          error: String(error),
          status: { ...s.status, connecting: false, owned_app_ids: null },
        }));
      return false;
    } finally {
      if (current === request) set({ busy: false });
    }
  },
}));
