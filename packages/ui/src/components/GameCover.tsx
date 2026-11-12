import { Package } from "lucide-react";
import { type CSSProperties, useState } from "react";
import { gameTitle } from "../hebrewTitle";
import { useTranslation } from "../i18n";
import type { GameSummary } from "../types";
import styles from "./GameBox.module.css";
export type GameBoxLanguage = "en" | "he";
export type GameBoxCover = { src: string; fallbackSrc?: string; title?: string };
const languages: GameBoxLanguage[] = ["en", "he"];
const cn = (...names: string[]) => names.filter(Boolean).join(" ");

/** Original Kitor six-face box geometry; interaction belongs to the containing tile/viewer. */
export function GameCover({
  game,
  language = game.coverLanguage ?? "he",
  rotation,
  zoom = 100,
}: {
  game: GameSummary;
  language?: GameBoxLanguage;
  rotation?: number;
  zoom?: number;
}) {
  const { language: appLanguage } = useTranslation();
  const covers = game.covers ?? { en: { src: game.cover ?? "" }, he: { src: game.cover ?? "" } };
  const title = gameTitle(game, appLanguage);
  return (
    <span
      className={styles.stage}
      data-game-cover={game.id}
      aria-hidden="true"
      data-language={language}
      data-rotatable={rotation !== undefined}
      style={
        {
          "--turn": `${rotation ?? (language === "he" ? 163 : -17)}deg`,
          "--zoom": zoom / 100,
        } as CSSProperties
      }
    >
      <span className={styles.shadow} />
      <BoxFaces title={title} covers={covers} />
    </span>
  );
}

function BoxFaces({
  title,
  covers,
}: {
  title: string;
  covers: Record<GameBoxLanguage, GameBoxCover>;
}) {
  return (
    <span className={styles.box} aria-hidden="true">
      {languages.map((side) => (
        <span key={side} className={cn(styles.face, styles.cover, styles[side])}>
          <BoxArtwork
            key={covers[side].src}
            src={covers[side].src}
            fallbackSrc={covers[side].fallbackSrc}
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
  );
}

function BoxArtwork({ src, fallbackSrc, title }: GameBoxCover & { title: string }) {
  const [loaded, setLoaded] = useState(false);
  const [failed, setFailed] = useState(false);

  const imageSrc = failed ? fallbackSrc : src;

  return (
    <>
      <span className={styles.placeholder}>
        <Package size={36} />
        <span dir="auto">{title}</span>
      </span>
      {imageSrc && (
        <img
          className={styles.art}
          data-loaded={loaded}
          src={imageSrc}
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
