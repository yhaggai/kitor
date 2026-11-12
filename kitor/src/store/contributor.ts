import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { create } from "zustand";
export type Contributor = { id: string; name: string };
let attempt = 0;
export const useContributor = create<{
  user: Contributor | null;
  pending: boolean;
  error: string;
  refresh: () => Promise<void>;
  signIn: () => Promise<void>;
  cancel: () => Promise<void>;
  signOut: () => Promise<void>;
}>((set) => ({
  user: null,
  pending: false,
  error: "",
  refresh: async () => {
    const current = attempt;
    try {
      const user = await invoke<Contributor | null>("contributor_status");
      if (current === attempt) set({ user, error: "" });
    } catch (e) {
      if (current === attempt) set({ user: null, error: String(e) });
    }
  },
  signIn: async () => {
    const current = ++attempt;
    set({ pending: true, error: "" });
    try {
      const url = await invoke<string>("contributor_sign_in");
      if (current !== attempt) return;
      await openUrl(url);
      for (let i = 0; i < 300 && current === attempt; i++) {
        await new Promise((resolve) => setTimeout(resolve, 2000));
        if (current !== attempt) return;
        const user = await invoke<Contributor | null>("contributor_poll");
        if (current !== attempt) return;
        if (user) {
          set({ user, pending: false });
          return;
        }
      }
      if (current === attempt) {
        await invoke("contributor_cancel");
        set({ pending: false, error: "Sign-in expired. Try again." });
      }
    } catch (e) {
      if (current === attempt) {
        set({ pending: false, error: String(e) });
        await invoke("contributor_cancel").catch(() => {});
      }
    }
  },
  cancel: async () => {
    ++attempt;
    set({ pending: false, error: "" });
    await invoke("contributor_cancel");
    await useContributor.getState().refresh();
  },
  signOut: async () => {
    ++attempt;
    try {
      await invoke("contributor_sign_out");
      set({ user: null, pending: false, error: "" });
    } catch (e) {
      set({ error: String(e) });
    }
  },
}));
