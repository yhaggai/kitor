import type { GameSummary } from "@kitor/ui";
import { convertFileSrc } from "@tauri-apps/api/core";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import covers from "@/data/game-box-covers.json";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function getCoverSrc(coverUrl: string | null | undefined): string | null {
  if (!coverUrl) return null;

  // Remote URLs
  if (coverUrl.startsWith("http://") || coverUrl.startsWith("https://")) {
    return coverUrl;
  }

  // Older catalogs can still contain paths to artwork removed from public/.
  if (
    /^\/[^/]+$/.test(coverUrl) ||
    (!coverUrl.startsWith("/") && !coverUrl.includes("\\") && !coverUrl.includes(":"))
  ) {
    return null;
  }

  // Local filesystem paths need conversion
  const normalizedPath = coverUrl.split("\\").join("/");
  return convertFileSrc(normalizedPath);
}

export function getCatalogCovers(id: string): GameSummary["covers"] {
  const cover = (covers as Record<string, GameSummary["covers"]>)[id];
  if (!cover) return undefined;

  const desktopCover = (side: typeof cover.en) => ({
    ...side,
    src: getCoverSrc(side.src) ?? "",
    fallbackSrc: getCoverSrc(side.fallbackSrc) ?? undefined,
  });
  return { en: desktopCover(cover.en), he: desktopCover(cover.he) };
}
