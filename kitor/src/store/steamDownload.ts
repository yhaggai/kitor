import { invoke } from "@tauri-apps/api/core";
import { ask } from "@tauri-apps/plugin-dialog";
import type { Game } from "./store";

export async function downloadSteamGames(
  appId: number,
  gameIds: string[],
  installRoot: string
): Promise<Game[] | null> {
  const download = (overwrite: boolean) =>
    invoke<Game[]>("download_steam_windows_game", { appId, gameIds, installRoot, overwrite });

  try {
    return await download(false);
  } catch (error) {
    const message = String(error);
    const prefix = "OVERWRITE_REQUIRED:";
    if (!message.startsWith(prefix)) throw error;

    const path = message.slice(prefix.length);
    const confirmed = await ask(
      `The installation folder already exists:\n${path}\n\nContinuing will replace this folder and delete any saves, patches, or other files inside it.`,
      {
        title: "Replace existing installation?",
        kind: "warning",
        okLabel: "Replace folder",
        cancelLabel: "Cancel",
      }
    );
    return confirmed ? download(true) : null;
  }
}
