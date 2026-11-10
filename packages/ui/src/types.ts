/** Presentation data only. Native game and patch models stay in the desktop app. */
export type GameStatus = "ready" | "available" | "unowned" | "attention" | "original";
export type Collection = "library" | "discovery";
export type GameSummary = {
  id: string;
  title: string;
  titleHe?: string;
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
};
