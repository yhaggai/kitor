export type PrimaryGameAction = "buy" | "install" | "install-patch" | "play" | "unavailable";

export function primaryGameAction(state: {
  inLibrary: boolean;
  filesReady: boolean;
  patched: boolean;
  patchAvailable: boolean;
  purchased: boolean;
}): PrimaryGameAction {
  if (state.inLibrary) {
    if (!state.filesReady) return "unavailable";
    if (state.patched || !state.patchAvailable) return "play";
    return "install-patch";
  }
  return state.purchased ? "install" : "buy";
}

export function ownsGame(input: {
  steamAppId?: number | null;
  gogId?: number | null;
  ownedSteamAppIds?: number[] | null;
  ownedGogIds?: number[];
}): boolean {
  const ownsSteam =
    input.steamAppId != null && (input.ownedSteamAppIds?.includes(input.steamAppId) ?? false);
  const ownsGog = input.gogId != null && (input.ownedGogIds?.includes(input.gogId) ?? false);
  return ownsSteam || ownsGog;
}

export function gogStoreUrl(installerPath?: string | null): string | null {
  const slug = installerPath?.match(/gog\.com\/downloads\/([^/?#]+)/i)?.[1];
  return slug ? `https://www.gog.com/en/game/${slug}` : null;
}
