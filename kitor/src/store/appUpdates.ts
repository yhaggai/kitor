import { Channel, invoke } from "@tauri-apps/api/core";
import { useStore } from "zustand";
import { type CheckResult, createAppUpdateStore, type UpdateProgress } from "@/lib/appUpdates";

export const appUpdateStore = createAppUpdateStore({
  check: () => invoke<CheckResult>("check_app_update"),
  install: async (version, onProgress) => {
    const progress = new Channel<UpdateProgress>();
    progress.onmessage = onProgress;
    await invoke("install_app_update", { version, progress });
  },
  restart: () => invoke("restart_after_app_update"),
});
export const useAppUpdates = () => useStore(appUpdateStore);
