import { invoke } from "@tauri-apps/api/core";
import { useSyncExternalStore } from "react";
import { parseDescriptionFeed } from "./descriptionFeedSchema";

const cacheKey = "kitor:game-descriptions:v1";
const listeners = new Set<() => void>();
let descriptions = loadCache();
let refreshInProgress: Promise<void> | undefined;

function loadCache() {
  try {
    return parseDescriptionFeed(localStorage.getItem(cacheKey) ?? "");
  } catch {
    return new Map<string, string>();
  }
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function usePublishedDescription(gameId: string | undefined) {
  return useSyncExternalStore(subscribe, () => (gameId ? descriptions.get(gameId) : undefined));
}

export function refreshDescriptionFeed() {
  refreshInProgress ??= (async () => {
    try {
      const json = await invoke<string>("fetch_game_descriptions");
      descriptions = parseDescriptionFeed(json);
      try {
        localStorage.setItem(cacheKey, json);
      } catch {
        // The current session can still use the update if storage is unavailable.
      }
      for (const listener of listeners) listener();
    } catch {
      // Keep the last validated feed when offline or when the response is invalid.
    }
  })().finally(() => {
    refreshInProgress = undefined;
  });

  return refreshInProgress;
}
