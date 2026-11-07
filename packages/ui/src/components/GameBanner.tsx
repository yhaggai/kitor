import { useState } from "react";
import { useTranslation } from "../i18n";
import type { GameSummary } from "../types";

export function GameBanner({ game }: { game: GameSummary }) {
  const { t } = useTranslation();
  const [language, setLanguage] = useState<"he" | "en">(game.banner?.he ? "he" : "en");
  const [failedSrc, setFailedSrc] = useState<string>();
  const activeLanguage = game.banner?.[language] ? language : game.banner?.he ? "he" : "en";
  const src = game.banner?.[activeLanguage];
  const failed = !!src && failedSrc === src;
  return (
    <div
      className={`kt-game-banner ${!src || failed ? "kt-game-banner--fallback" : ""}`}
      style={{ aspectRatio: game.banner?.aspectRatio }}
    >
      {src && !failed ? (
        <>
          <img
            className="kt-game-banner-art"
            src={src}
            alt={t("{0} · {1} banner", [
              game.title,
              activeLanguage === "he" ? t("Hebrew") : t("English"),
            ])}
            onError={() => setFailedSrc(src)}
          />
          {activeLanguage === "en" && game.banner?.logo && (
            <img className="kt-game-banner-logo" src={game.banner.logo} alt="" />
          )}
        </>
      ) : (
        <div className="kt-banner-fallback-title" dir="auto">
          {game.title}
        </div>
      )}
      {game.banner?.he && game.banner?.en && (
        <fieldset className="kt-banner-language" aria-label={t("Banner language")}>
          <button
            type="button"
            aria-pressed={activeLanguage === "he"}
            onClick={() => {
              setLanguage("he");
              setFailedSrc(undefined);
            }}
          >
            עברית
          </button>
          <button
            type="button"
            aria-pressed={activeLanguage === "en"}
            onClick={() => {
              setLanguage("en");
              setFailedSrc(undefined);
            }}
          >
            {t("English")}{" "}
          </button>
        </fieldset>
      )}
    </div>
  );
}
