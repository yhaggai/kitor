import { useTranslation } from "@kitor/ui";
import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { type MouseEvent, useState } from "react";
import { gameDescription, translationCommunity } from "@/lib/gameDescriptions";

const linkClass = "text-primary underline underline-offset-4";

export function TranslationCredit({ gameId }: { gameId: string }) {
  const { t } = useTranslation();
  const [error, setError] = useState(false);
  const { communityTranslation, sourceUrl } = gameDescription(gameId);
  if (!communityTranslation) return null;

  function openExternal(event: MouseEvent<HTMLAnchorElement>) {
    if (!isTauri()) return;
    event.preventDefault();
    setError(false);
    void openUrl(event.currentTarget.href).catch(() => setError(true));
  }

  return (
    <div className="mb-4 space-y-1 text-sm">
      <p>
        {t("Translation by")}{" "}
        <a
          href={translationCommunity.url}
          target="_blank"
          rel="noopener noreferrer"
          className={linkClass}
          onClick={openExternal}
          lang="he"
          dir="rtl"
        >
          {translationCommunity.name}
        </a>
      </p>
      {sourceUrl && (
        <p>
          <a
            href={sourceUrl}
            target="_blank"
            rel="noopener noreferrer"
            className={linkClass}
            onClick={openExternal}
          >
            {t("Original Facebook post")}
          </a>
        </p>
      )}
      {error && <p role="alert">{t("Could not open the link. Please try again.")}</p>}
    </div>
  );
}
