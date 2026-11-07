import catalog from "@/data/game-descriptions.he.json";

type Description = {
  description: string;
  metadataSource?: string;
  sourceUrl: string;
  communityTranslation: boolean;
};
export const translationCommunity = {
  name: "הרפתקה עברית",
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
