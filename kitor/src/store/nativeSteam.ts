import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import { useSteamStore } from "./steam";

type AuthEvent =
  | { kind: "qr"; svg: string }
  | { kind: "connected"; account: string }
  | { kind: "auth_challenge"; code_type: number | null; message: string };
type PasswordLogin = { username: string; password: string };
type Method = "qr" | "password";
let pending: Promise<boolean> | null = null;
let active: Promise<void> | null = null;
let settle: ((connected: boolean) => void) | null = null;
let cancelled = false;
function finish(connected: boolean) {
  const resolve = settle;
  pending = null;
  settle = null;
  resolve?.(connected);
}
export const useNativeSteam = create<{
  connected: boolean;
  loading: boolean;
  busy: boolean;
  changing: boolean;
  open: boolean;
  method: Method;
  qr: string | null;
  error: string | null;
  challenge: { code_type: number | null; message: string } | null;
  submittingCode: boolean;
  refresh: () => Promise<boolean>;
  ensureConnected: () => Promise<boolean>;
  retry: (credentials?: PasswordLogin) => Promise<void>;
  switchMethod: (method: Method) => Promise<void>;
  submitCode: (code: string) => Promise<void>;
  cancel: () => Promise<void>;
  disconnect: () => Promise<void>;
}>((set, get) => ({
  connected: false,
  loading: true,
  busy: false,
  changing: false,
  open: false,
  method: "qr",
  qr: null,
  error: null,
  challenge: null,
  submittingCode: false,
  refresh: async () => {
    try {
      const connected = await invoke<boolean>("get_native_steam_status");
      set({ connected, loading: false });
      return connected;
    } catch (e) {
      set({ error: String(e), loading: false });
      return false;
    }
  },
  ensureConnected: () => {
    if (pending) return pending;
    if (get().busy) return Promise.resolve(false);
    if (get().connected) return Promise.resolve(true);
    pending = new Promise((resolve) => {
      settle = resolve;
    });
    const result = pending;
    void (async () => {
      if (await get().refresh()) {
        finish(true);
        return;
      }
      set({ open: true, error: null, method: "qr" });
    })();
    return result;
  },
  retry: (credentials) => {
    if (get().busy || get().changing) return Promise.resolve();
    if (get().method === "password" && !credentials) return Promise.resolve();
    cancelled = false;
    set({ busy: true, error: null, qr: null, challenge: null, submittingCode: false });
    active = (async () => {
      let unlisten: (() => void) | undefined;
      try {
        unlisten = await listen<AuthEvent>("native-steam-auth", ({ payload }) => {
          if (cancelled) return;
          if (payload.kind === "qr")
            set({ qr: `data:image/svg+xml;charset=utf-8,${encodeURIComponent(payload.svg)}` });
          if (payload.kind === "auth_challenge")
            set({ challenge: payload, submittingCode: false, error: null });
        });
        if (cancelled) return;
        const login = credentials
          ? invoke("connect_native_steam_password", credentials)
          : invoke("connect_native_steam");
        credentials = undefined;
        await login;
        if (!cancelled) {
          useSteamStore.getState().reset();
          set({ connected: true, open: false, loading: false });
          void useSteamStore.getState().run("refresh_steam_library");
          finish(true);
        }
      } catch (e) {
        if (!cancelled) set({ error: String(e) });
      } finally {
        unlisten?.();
        set({ busy: false, qr: null, challenge: null, submittingCode: false });
      }
    })();
    return active;
  },
  switchMethod: async (method) => {
    if (get().changing || get().method === method) return;
    set({ changing: true });
    cancelled = true;
    try {
      await invoke("cancel_native_steam_login");
      await active;
      set({ method, error: null, qr: null, challenge: null });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ changing: false });
    }
    if (get().open && method === "qr" && get().method === "qr") await get().retry();
  },
  submitCode: async (code) => {
    if (get().submittingCode) return;
    set({ submittingCode: true, error: null });
    try {
      await invoke("submit_native_steam_guard", { code });
    } catch (e) {
      set({ error: String(e), submittingCode: false });
    }
  },
  cancel: async () => {
    cancelled = true;
    set({ open: false, qr: null, error: null });
    try {
      await invoke("cancel_native_steam_login");
      await active;
    } catch (e) {
      set({ error: String(e) });
    }
    finish(false);
  },
  disconnect: async () => {
    set({ busy: true, error: null });
    try {
      await invoke("disconnect_native_steam");
      useSteamStore.getState().reset();
      set({ connected: false });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ busy: false });
    }
  },
}));
