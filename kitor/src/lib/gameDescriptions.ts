import catalog from "@/data/game-descriptions.he.json";

type Description = {
  description: string;
  metadataSource?: string;
  originalReleaseDate?: string;
  sourceUrl: string;
  communityTranslation: boolean;
};
const releaseMetadata = import.meta.glob<string | null>(
  "../../../patches-metadata/[0-9]*.json",
  { eager: true, import: "original_release_date" }
);

export function gameReleaseDate(id: string): string | undefined {
  const { metadataSource, originalReleaseDate } = gameDescription(id);
  const releaseDate = metadataSource ? releaseMetadata[`../../../${metadataSource}`] : undefined;
  return releaseDate ?? originalReleaseDate;
}

export const translationCommunity = {
  name: "הרפתקאה בעברית",
  url: "https://www.facebook.com/groups/hebrewadventure",
};

export function gameDescription(id: string): Description {
  return (
    (catalog as Record<string, Description>)[id] ?? {
      description:
        "הרפתקה חדשה מחכה לכם. חקרו את עולם המשחק, הכירו את הדמויות וגלו לאן יוביל אתכם הסיפור.",
      sourceUrl: "",
      communityTranslation: false,
    }
  );
}
