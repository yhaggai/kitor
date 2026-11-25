import type { Tag } from "@kitor/registry";

/** Presentation data only. Native game and patch models stay in the desktop app. */
export type GameStatus = "ready" | "available" | "unowned" | "attention" | "original";
export type Collection = "library" | "discovery";
export type Studio = { id: string; name: string; logo?: string; logoOnDark?: boolean };
export type GameSummary = {
  id: string;
  title: string;
  titleHe?: string;
  originalDevelopers?: Studio[];
  publishers?: string[];
  cover?: string;
  coverLanguage?: "en" | "he";
  banner?: { en?: string; he?: string; logo?: string; aspectRatio?: number };
  covers?: Record<"en" | "he", { src: string; fallbackSrc?: string; title?: string }>;
  screenshots?: { src: string; alt?: string; sourceUrl?: string }[];
  description?: string;
  searchTerms?: string;
  status: GameStatus;
  language?: string;
  author?: string;
  version?: string;
  /** Registry keywords. A game may belong to more than one category. */
  tags?: Tag[];
  translationBeta?: boolean;
  /** Date first added to the Kitor registry, in YYYY-MM-DD format. */
  addedOn?: string;
  /** Earliest known release, in YYYY, YYYY-MM, or YYYY-MM-DD format. */
  originalReleaseDate?: string;
};
