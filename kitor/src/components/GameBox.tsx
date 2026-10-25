import { Package } from "lucide-react";
import { type CSSProperties, useState } from "react";
import { cn } from "@/lib/utils";
import styles from "./GameBox.module.css";

export type GameBoxLanguage = "en" | "he";

export type GameBoxCover = {
  /** A browser-ready image URL. Convert native paths with getCoverSrc first. */
  src: string;
  /** Localized title, also printed on this side's spine. */
  title?: string;
};

export type GameBoxProps = {
  title: string;
  covers: Record<GameBoxLanguage, GameBoxCover>;
  /** Controls the resting side; hover temporarily shows the opposite side. */
  language?: GameBoxLanguage;
  defaultLanguage?: GameBoxLanguage;
  /** Called for explicit selections and flips, never for temporary hover previews. */
  onLanguageChange?: (language: GameBoxLanguage) => void;
  showLanguageControls?: boolean;
  className?: string;
  style?: CSSProperties;
};

const languageLabels: Record<GameBoxLanguage, string> = { en: "English", he: "עברית" };
const languageNames: Record<GameBoxLanguage, string> = { en: "English", he: "Hebrew" };
const languages: GameBoxLanguage[] = ["en", "he"];
const opposite = (language: GameBoxLanguage): GameBoxLanguage => (language === "en" ? "he" : "en");

/** A responsive 3D cover. Hover previews; click, tap, Space, or Enter flips the resting side. */
export function GameBox({
  title,
  covers,
  language,
  defaultLanguage = "en",
  onLanguageChange,
  showLanguageControls = true,
  className,
  style,
}: GameBoxProps) {
  const [selectedLanguage, setSelectedLanguage] = useState(defaultLanguage);
  const [hovering, setHovering] = useState(false);
  const restingLanguage = language ?? selectedLanguage;
  const visibleLanguage = hovering ? opposite(restingLanguage) : restingLanguage;
  const nextLanguage = opposite(visibleLanguage);

  const selectLanguage = (next: GameBoxLanguage) => {
    setHovering(false);
    if (language === undefined) setSelectedLanguage(next);
    if (next !== restingLanguage) onLanguageChange?.(next);
  };

  return (
    <section
      className={cn(styles.root, className)}
      style={style}
      aria-label={`${title} bilingual game box`}
      dir="ltr"
    >
      <div className={styles.header}>
        <span className={styles.title} dir="auto">
          {title}
        </span>
        {showLanguageControls && (
          <fieldset className={styles.languages} aria-label="Starting language">
            {languages.map((side) => (
              <button
                key={side}
                type="button"
                className={styles.language}
                lang={side}
                dir={side === "he" ? "rtl" : "ltr"}
                aria-pressed={restingLanguage === side}
                onClick={() => selectLanguage(side)}
              >
                {languageLabels[side]}
              </button>
            ))}
          </fieldset>
        )}
      </div>
      <div className={styles.stage} data-hovered={hovering}>
        <div className={styles.halo} aria-hidden="true" />
        <div className={styles.shadow} aria-hidden="true" />
        <button
          type="button"
          className={styles.target}
          data-language={visibleLanguage}
          aria-label={`${title}: ${languageNames[visibleLanguage]} cover. Show ${languageNames[nextLanguage]} cover`}
          onPointerEnter={(event) => {
            if (event.pointerType === "mouse" || event.pointerType === "pen") setHovering(true);
          }}
          onPointerLeave={() => setHovering(false)}
          onPointerCancel={() => setHovering(false)}
          onClick={() => selectLanguage(nextLanguage)}
        >
          <span className={styles.box} aria-hidden="true">
            {languages.map((side) => (
              <span key={side} className={cn(styles.face, styles.cover, styles[side])}>
                <BoxArtwork
                  key={covers[side].src}
                  src={covers[side].src}
                  title={covers[side].title ?? title}
                />
              </span>
            ))}
            {languages.map((side) => (
              <span key={side} className={cn(styles.face, styles.spine, styles[`spine_${side}`])}>
                <span className={styles.spineMark}>PC</span>
                <span className={styles.spineTitle} lang={side} dir={side === "he" ? "rtl" : "ltr"}>
                  {covers[side].title ?? title}
                </span>
                <span className={styles.spineMark}>{side.toUpperCase()}</span>
              </span>
            ))}
            <span className={cn(styles.face, styles.lid)} />
            <span className={cn(styles.face, styles.lid, styles.bottom)} />
          </span>
        </button>
      </div>
    </section>
  );
}

function BoxArtwork({ src, title }: { src: string; title: string }) {
  const [loaded, setLoaded] = useState(false);
  const [failed, setFailed] = useState(false);

  return (
    <>
      <span className={styles.placeholder}>
        <Package size={36} />
        <span dir="auto">{title}</span>
      </span>
      {src && !failed && (
        <img
          className={styles.art}
          data-loaded={loaded}
          src={src}
          alt=""
          draggable={false}
          decoding="async"
          onLoad={() => setLoaded(true)}
          onError={() => setFailed(true)}
        />
      )}
    </>
  );
}
