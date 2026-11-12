import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import type { PatchDownloadProgress } from "@/hooks/useGameCardState";
import { type Game, type GameHealth, useAppStore } from "./store";

type InstallProgress = {
  game_id: string;
  step: string;
  phase: "download" | "install";
  downloaded_bytes?: number | null;
  total_bytes?: number | null;
  percentage: number | null;
};
export const useGogDownload = create<{
  phase: "download" | "install" | "patch";
  installedGameId: number | null;
  busy: boolean;
  patchDownload: PatchDownloadProgress | null;
  downloadedBytes: number | null;
  totalBytes: number | null;
  gameId: string | null;
  error: string | null;
  complete: boolean;
  step: string;
  percentage: number | null;
  start: (
    source: number | { steamAppId: number } | { installedGameId: number },
    gameId: string,
    installRoot?: string
  ) => Promise<void>;
}>((set, get) => ({
  phase: "download",
  installedGameId: null,
  busy: false,
  patchDownload: null,
  downloadedBytes: null,
  totalBytes: null,
  gameId: null,
  error: null,
  complete: false,
  step: "",
  percentage: null,
  start: async (source, gameId, installRoot) => {
    if (get().busy) return;
    const account = useAppStore.getState();
    if (
      typeof source === "number" &&
      (!account.gogAuth.isAuthenticated || !account.gogGames.some((game) => game.id === source))
    ) {
      set({
        gameId,
        error: "Connect the GOG account that owns this game, then retry.",
        complete: false,
      });
      return;
    }
    set({
      phase: typeof source !== "number" && "installedGameId" in source ? "install" : "download",
      installedGameId: null,
      busy: true,
      patchDownload: null,
      downloadedBytes: null,
      totalBytes: null,
      gameId,
      error: null,
      complete: false,
      step: "Starting installation…",
      percentage: null,
    });
    const listeners: (() => void)[] = [];
    try {
      const root = installRoot ?? (await invoke<string>("get_game_install_root"));
      let game: Game;
      if (typeof source !== "number" && "installedGameId" in source) {
        const games = await invoke<Game[]>("get_games_with_patches");
        const installed = games.find(
          (entry) => entry.id === source.installedGameId && entry.scummvm_id === gameId
        );
        if (!installed) throw new Error("Installed game was not found in the library.");
        game = installed;
      } else if (typeof source === "number") {
        listeners.push(
          await listen<InstallProgress>("gog-game-install-progress", ({ payload }) => {
            if (payload.game_id === gameId)
              set({
                phase: payload.phase,
                step: payload.step,
                percentage: payload.percentage,
                downloadedBytes: payload.downloaded_bytes ?? null,
                totalBytes: payload.total_bytes ?? null,
              });
          })
        );
        game = await invoke<Game>("install_gog_game", {
          gameId: source,
          scummvmId: gameId,
          installRoot: root,
        });
      } else {
        listeners.push(
          await listen<string>("steam-game-importing", ({ payload }) => {
            if (payload === gameId) set({ phase: "install", percentage: null });
          })
        );
        listeners.push(
          await listen<{ kind: string; downloaded_bytes?: number; total_bytes?: number }>(
            "native-steam-download",
            ({ payload }) => {
              if (payload.kind === "progress")
                set({
                  phase: "download",
                  downloadedBytes: payload.downloaded_bytes ?? null,
                  totalBytes: payload.total_bytes ?? null,
                  percentage: payload.total_bytes
                    ? Math.min(100, ((payload.downloaded_bytes ?? 0) / payload.total_bytes) * 100)
                    : null,
                });
            }
          )
        );
        const imported = await invoke<Game[]>("download_steam_windows_game", {
          appId: source.steamAppId,
          gameIds: [gameId],
          installRoot: root,
        });
        const selected = imported.find((entry) => entry.scummvm_id === gameId);
        if (!selected) throw new Error("Installed game was not found in the library.");
        game = selected;
      }
      const store = useAppStore.getState();
      set({
        phase: "install",
        percentage: null,
        step: "Preparing game runtime…",
        installedGameId: game.id,
      });
      const games = await invoke<Game[]>("get_games_with_patches");
      const installed = games.find((entry) => entry.id === game.id);
      if (!installed) throw new Error("Installed game was not found in the library.");
      if (installed.requires_scummvm !== false) {
        const status = await invoke<{ downloaded: boolean }>("check_scummvm_status");
        if (!status.downloaded) await invoke("download_scummvm_binary");
        store.setScummVMReady(true);
      }
      set({
        phase: "patch",
        percentage: null,
        step: "Installing the Hebrew patch",
      });
      const patch =
        installed.available_patches?.find((entry) => entry.language === "he") ??
        installed.available_patches?.find((entry) => entry.language.startsWith("he-"));
      if (!patch) throw new Error("No compatible Hebrew patch is available for this game.");
      listeners.push(
        await listen<{ scummvm_id: string; progress: number; step: string }>(
          "install-progress",
          ({ payload }) => {
            if (payload.scummvm_id === gameId)
              set({ percentage: payload.progress, step: payload.step });
          }
        )
      );
      listeners.push(
        await listen<PatchDownloadProgress>("patch-download-progress", ({ payload }) => {
          if (payload.scummvm_id === gameId) set({ patchDownload: payload });
        })
      );
      const recordedPatch = installed.installed_patches?.find(
        (entry) =>
          entry.language === patch.language &&
          entry.version === patch.version &&
          entry.release_revision === patch.release_revision &&
          entry.status === "active"
      );
      let patchVerified = false;
      if (recordedPatch) {
        const health = await invoke<GameHealth>("refresh_game", { gameId: game.id });
        patchVerified = health.patch_status === "installed";
        if (!patchVerified) {
          store.setGames(
            games.map((entry) => (entry.id === game.id ? { ...entry, health } : entry))
          );
          // The restore command validates the backup and refuses to overwrite conflicting changes.
          await invoke("restore_backup", { installedPatchId: recordedPatch.id });
        }
      }
      if (!patchVerified) {
        await invoke("download_and_install_patch", {
          scummvmId: gameId,
          language: patch.language,
          version: patch.version,
        });
      }
      store.setGames(await invoke<Game[]>("get_games_with_patches"));
      store.setHighlightedGameId(game.id);
      set({ complete: true, step: "Ready to play", percentage: 100 });
    } catch (error) {
      set({ error: String(error) });
    } finally {
      for (const stop of listeners) stop();
      set({ busy: false });
    }
  },
}));
