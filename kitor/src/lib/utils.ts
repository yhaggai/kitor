import { convertFileSrc } from "@tauri-apps/api/core";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function getCoverSrc(coverUrl: string | null | undefined): string | null {
  if (!coverUrl) return null;

  // Remote URLs
  if (coverUrl.startsWith("http://") || coverUrl.startsWith("https://")) {
    return coverUrl;
  }

  // Public folder assets can be referenced directly
  if (coverUrl.startsWith("/") || (!coverUrl.includes("\\") && !coverUrl.includes(":"))) {
    return coverUrl;
  }

  // Local filesystem paths need conversion
  const normalizedPath = coverUrl.split("\\").join("/");
  return convertFileSrc(normalizedPath);
}
