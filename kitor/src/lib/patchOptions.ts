import { PatchLanguage } from "@kitor/registry";
import type { AvailablePatchInfo, InstalledPatchInfo } from "@/store/store";

export function patchOptionLabel(language: PatchLanguage | "", gameId?: string): string {
  if (language === PatchLanguage.Hebrew)
    return gameId === "undertale" ? "Hebrew without nikud" : "Hebrew";
  if (language === PatchLanguage.HebrewNiqqud) return "Hebrew with nikud";
  return language;
}

export function selectedPatchOption(
  available: AvailablePatchInfo[],
  installed: InstalledPatchInfo[],
  choice: PatchLanguage | ""
) {
  const language = choice || installed.find((patch) => patch.status === "active")?.language;
  return available.find((patch) => patch.language === language) ?? available[0];
}
