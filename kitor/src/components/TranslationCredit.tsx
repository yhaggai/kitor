import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { gameDescription, translationCommunity } from "@/lib/gameDescriptions";

export function TranslationCredit({ gameId }: { gameId: string }) {
  const [error, setError] = useState(false);
  if (!gameDescription(gameId).communityTranslation) return null;
  return (
    <div dir="rtl" lang="he" className="mt-4 text-sm text-muted-foreground">
      <p>
        התרגום לעברית והתיאור באדיבות קהילת{" "}
        <a
          href={translationCommunity.url}
          target="_blank"
          rel="noopener noreferrer"
          className="text-primary underline underline-offset-4"
          onClick={(event) => {
            event.preventDefault();
            setError(false);
            void openUrl(translationCommunity.url).catch(() => setError(true));
          }}
        >
          הרפתקה עברית
        </a>{" "}
        וחברי צוות התרגום. תודה על ההשקעה!
      </p>
      {error && <p role="alert">לא ניתן לפתוח את הקישור כעת. אפשר לנסות שוב.</p>}
    </div>
  );
}
