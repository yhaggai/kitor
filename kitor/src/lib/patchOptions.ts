import type { AvailablePatchInfo, InstalledPatchInfo } from "@/store/store";

export function patchOptionLabel(language: string, gameId?: string): string {
  if (language === "he") return gameId === "undertale" ? "Hebrew without nikud" : "Hebrew";
  if (language === "he-x-nikud") return "Hebrew with nikud";
  return language;
}

export function selectedPatchOption(
  available: AvailablePatchInfo[],
  installed: InstalledPatchInfo[],
  choice: string
) {
  const language = choice || installed.find((patch) => patch.status === "active")?.language;
  return available.find((patch) => patch.language === language) ?? available[0];
}
