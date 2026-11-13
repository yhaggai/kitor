type LibraryProvider = "steam" | "gog";

export function refreshLibraryOnStartup(provider: LibraryProvider): boolean {
  try {
    return localStorage.getItem(`kitor-${provider}-refresh-on-startup`) === "true";
  } catch {
    return false;
  }
}

export function setRefreshLibraryOnStartup(provider: LibraryProvider, enabled: boolean) {
  localStorage.setItem(`kitor-${provider}-refresh-on-startup`, String(enabled));
}
