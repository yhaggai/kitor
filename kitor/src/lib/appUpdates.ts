import { createStore } from "zustand/vanilla";

export type UpdateInfo = { version: string; notes: string };
export type CheckResult = {
  currentVersion: string;
  canInstall: boolean;
  update: UpdateInfo | null;
};
export type UpdateProgress =
  | { event: "downloading"; downloaded: number; total: number | null }
  | { event: "verifying" }
  | { event: "installing" };
export type UpdateTransport = {
  check: () => Promise<CheckResult>;
  install: (version: string, onProgress: (progress: UpdateProgress) => void) => Promise<void>;
  restart: () => Promise<void>;
};
export type UpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "upToDate"
  | "downloading"
  | "verifying"
  | "installing"
  | "ready"
  | "error";
export const isInstallingUpdate = (status: UpdateStatus) =>
  ["downloading", "verifying", "installing", "ready"].includes(status);

type UpdateState = {
  status: UpdateStatus;
  currentVersion: string | null;
  canInstall: boolean;
  update: UpdateInfo | null;
  error: string | null;
  downloaded: number;
  total: number | null;
  open: boolean;
  check: (manual?: boolean) => Promise<void>;
  install: () => Promise<void>;
  restart: () => Promise<void>;
  setOpen: (open: boolean) => void;
};

export function createAppUpdateStore(transport: UpdateTransport) {
  let checkedAutomatically = false;
  let restarting = false;
  return createStore<UpdateState>((set, get) => ({
    status: "idle",
    currentVersion: null,
    canInstall: false,
    update: null,
    error: null,
    downloaded: 0,
    total: null,
    open: false,
    setOpen: (open) => {
      if (!open && isInstallingUpdate(get().status)) return;
      set({ open });
    },
    check: async (manual = false) => {
      if (manual) set({ open: true });
      if (get().status === "checking" || isInstallingUpdate(get().status)) return;
      if (!manual && checkedAutomatically) return;
      checkedAutomatically = true;
      set({ status: "checking", error: null, update: null });
      try {
        const result = await transport.check();
        set({
          ...result,
          status: result.update ? "available" : "upToDate",
          // A quiet check never interrupts the user for offline/error/no-update cases.
          open: get().open || Boolean(result.update),
        });
      } catch (error) {
        set({ status: "error", error: String(error) });
      }
    },
    install: async () => {
      const { update, status, canInstall } = get();
      if (!update || !canInstall || status !== "available") return;
      set({ status: "downloading", error: null, downloaded: 0, total: null, open: true });
      try {
        await transport.install(update.version, (progress) => {
          if (progress.event === "downloading") {
            set({ status: "downloading", downloaded: progress.downloaded, total: progress.total });
          } else {
            set({ status: progress.event });
          }
        });
        set({ status: "ready" });
        await get().restart();
      } catch (error) {
        // Require a fresh check after a failed download/install before retrying.
        set({ status: "error", error: String(error), update: null });
      }
    },
    restart: async () => {
      if (get().status !== "ready" || restarting) return;
      restarting = true;
      set({ error: null });
      try {
        await transport.restart();
      } catch {
        // Installation succeeded. Never download/reinstall just because relaunch failed.
        set({ error: "The update is installed. Restart Kitor to finish." });
      } finally {
        restarting = false;
      }
    },
  }));
}
