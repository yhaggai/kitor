import type { GameSummary } from "@kitor/ui";
import banners from "@/data/game-banners.json";
import covers from "@/data/game-box-covers.json";
import galleries from "@/data/game-galleries.json";
import { gameDescription } from "@/lib/gameDescriptions";
import { patchOptionLabel } from "@/lib/patchOptions";
import { getCoverSrc } from "@/lib/utils";
import type { Game, PatchWithStatus } from "@/store/store";

const classicTitles: Record<string, string> = {
  "monkey-se": "The Secret of Monkey Island — Classic",
  "monkey2-se": "Monkey Island 2: LeChuck's Revenge — Classic",
};

function artwork(
  id: string,
  cover?: string
): Pick<GameSummary, "cover" | "covers" | "banner" | "screenshots"> {
  return {
    screenshots: (galleries as Record<string, GameSummary["screenshots"]>)[id],
    banner: (banners as Record<string, GameSummary["banner"]>)[id],
    cover: getCoverSrc(cover) ?? undefined,
    covers: (covers as Record<string, GameSummary["covers"]>)[id],
  };
}

export function gameSummary(game: Game): GameSummary {
  const installed = game.installed_patches?.find((patch) => patch.status === "active");
  const patch =
    game.available_patches?.find((patch) => patch.language === installed?.language) ??
    game.available_patches?.[0];
  const unhealthy =
    game.health &&
    (game.health.game_status !== "present" ||
      ["damaged", "unverified"].includes(game.health.patch_status));
  const patched = game.health
    ? game.health.patch_status === "installed"
    : (game.is_patched ?? !!installed);
  return {
    id: String(game.id),
    searchTerms: `${game.scummvm_id} ${game.title} ${game.title_he ?? ""}`,
    title: classicTitles[game.scummvm_id] ?? game.title,
    titleHe: game.title_he ?? undefined,
    ...artwork(game.scummvm_id, game.cover_url),
    coverLanguage: "he",
    status: unhealthy
      ? "attention"
      : patched
        ? "ready"
        : game.has_available_patch
          ? "available"
          : "original",
    description: gameDescription(game.scummvm_id).description,
    author: gameDescription(game.scummvm_id).communityTranslation ? undefined : patch?.author,
    language: patch ? patchOptionLabel(patch.language, game.scummvm_id) : undefined,
    version: installed?.version ?? patch?.version,
  };
}

export function patchSummary(patch: PatchWithStatus): GameSummary {
  return {
    id: `${patch.scummvm_id}:${patch.language}:${patch.version}`,
    searchTerms: `${patch.scummvm_id} ${patch.game_title} ${patch.title_he ?? ""} ${(patch.tags ?? []).join(" ")}`,
    tags: patch.tags ?? [],
    title: classicTitles[patch.scummvm_id] ?? patch.game_title,
    titleHe: patch.title_he ?? undefined,
    ...artwork(patch.scummvm_id, patch.cover_url),
    coverLanguage: "he",
    status: patch.patch_installed ? "ready" : patch.game_installed ? "available" : "unowned",
    description: gameDescription(patch.scummvm_id).description,
    author: gameDescription(patch.scummvm_id).communityTranslation ? undefined : patch.author,
    version: patch.version,
    language: patchOptionLabel(patch.language, patch.scummvm_id),
  };
}
