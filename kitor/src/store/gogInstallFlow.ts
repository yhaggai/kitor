import { create } from "zustand";
import { useGogDownload } from "./gogDownload";

type Target = { gameId: string; installedGameId?: number } & (
  | { gogId: number; purchaseUrl?: string | null; steamAppId?: never }
  | { steamAppId: number; installRoot?: string; gogId?: never }
);
export const useGogInstallFlow = create<{
  target: Target | null;
  open: boolean;
  show: (target: Target) => void;
  setOpen: (open: boolean) => void;
}>((set, get) => ({
  target: null,
  open: false,
  show: (target) => {
    if (useGogDownload.getState().busy && get().target) {
      set({ open: true });
      return;
    }
    if (
      get().target?.gameId !== target.gameId ||
      get().target?.steamAppId !== target.steamAppId ||
      target.installedGameId !== undefined
    ) {
      useGogDownload.setState({
        complete: false,
        error: null,
        gameId: null,
        installedGameId: null,
      });
    }
    set({ target, open: true });
  },
  setOpen: (open) => {
    const download = useGogDownload.getState();
    if (!open && download.complete && get().target?.gameId === download.gameId) {
      set({ open: false, target: null });
      return;
    }
    set({ open });
  },
}));
