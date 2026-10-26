import * as Dialog from "@radix-ui/react-dialog";
import { Package, X, ZoomIn, ZoomOut } from "lucide-react";
import { type CSSProperties, useRef, useState } from "react";
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
  /** Language of viewer controls; cover language is independent. */
  locale?: "en" | "he";
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
const languages: GameBoxLanguage[] = ["en", "he"];
const opposite = (language: GameBoxLanguage): GameBoxLanguage => (language === "en" ? "he" : "en");

/** A responsive 3D cover with a draggable enlarged viewer. */
export function GameBox({
  title,
  locale = "en",
  covers,
  language,
  defaultLanguage = "en",
  onLanguageChange,
  showLanguageControls = true,
  className,
  style,
}: GameBoxProps) {
  const hebrew = locale === "he";
  const [selectedLanguage, setSelectedLanguage] = useState(defaultLanguage);
  const [hovering, setHovering] = useState(false);
  const restingLanguage = language ?? selectedLanguage;
  const visibleLanguage = hovering ? opposite(restingLanguage) : restingLanguage;
  const [open, setOpen] = useState(false);
  const [rotation, setRotation] = useState(-17);
  const [zoom, setZoom] = useState(100);
  const changeZoom = (delta: number) =>
    setZoom((value) => Math.min(200, Math.max(50, value + delta)));
  const drag = useRef<{ id: number; x: number; rotation: number } | null>(null);

  const selectLanguage = (next: GameBoxLanguage) => {
    setHovering(false);
    if (language === undefined) setSelectedLanguage(next);
    if (next !== restingLanguage) onLanguageChange?.(next);
  };

  return (
    <Dialog.Root
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) drag.current = null;
      }}
    >
      <section
        className={cn(styles.root, className)}
        style={style}
        aria-label={hebrew ? `קופסת המשחק ${title}` : `${title} bilingual game box`}
        dir="ltr"
      >
        <div className={styles.header}>
          <span className={styles.title} dir="auto">
            {title}
          </span>
          {showLanguageControls && (
            <fieldset className={styles.languages} aria-label={hebrew ? "שפת הקופסה" : "Starting language"}>
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
          <Dialog.Trigger asChild>
            <button
              type="button"
              className={styles.target}
              data-language={visibleLanguage}
              aria-label={hebrew ? `הגדלת קופסת המשחק ${title}` : `Enlarge ${title} game box`}
              onPointerEnter={(event) => {
                if (event.pointerType === "mouse" || event.pointerType === "pen") setHovering(true);
              }}
              onPointerLeave={() => setHovering(false)}
              onPointerCancel={() => setHovering(false)}
              onClick={() => {
                setRotation(visibleLanguage === "he" ? 163 : -17);
                setZoom(100);
                setHovering(false);
              }}
            >
              <BoxFaces title={title} covers={covers} />
            </button>
          </Dialog.Trigger>
        </div>
      </section>
      <Dialog.Portal>
        <Dialog.Overlay className={styles.overlay} />
        <Dialog.Content className={cn(styles.root, styles.viewer)} dir="ltr">
          <Dialog.Title className={styles.viewerTitle} dir="auto">
            {title}
          </Dialog.Title>
          <Dialog.Description className={styles.instructions}>
            {hebrew ? "גררו לצדדים או השתמשו במקשי החצים לסיבוב. השתמשו ב־+ / − להגדלה והקטנה." : "Hold and drag left or right to rotate. Use arrow keys to rotate and + / − to zoom."}
          </Dialog.Description>
          <div className={cn(styles.stage, styles.viewerStage)}>
            <div className={styles.halo} aria-hidden="true" />
            <div className={styles.shadow} aria-hidden="true" />
            <button
              type="button"
              className={cn(styles.target, styles.rotationTarget)}
              aria-label={hebrew ? "סיבוב קופסת המשחק" : "Rotate game box"}
              style={{ "--turn": `${rotation}deg`, "--zoom": zoom / 100 } as CSSProperties}
              onKeyDown={(event) => {
                if (["+", "=", "-", "0"].includes(event.key)) {
                  event.preventDefault();
                  if (event.key === "0") setZoom(100);
                  else changeZoom(event.key === "-" ? -25 : 25);
                }
                if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
                  event.preventDefault();
                  setRotation((angle) => angle + (event.key === "ArrowLeft" ? -15 : 15));
                }
              }}
              onPointerDown={(event) => {
                if (!event.isPrimary || event.button !== 0 || drag.current) return;
                event.currentTarget.focus();
                event.currentTarget.setPointerCapture(event.pointerId);
                drag.current = { id: event.pointerId, x: event.clientX, rotation };
              }}
              onPointerMove={(event) => {
                if (drag.current?.id !== event.pointerId) return;
                setRotation(drag.current.rotation + (event.clientX - drag.current.x) * 0.5);
              }}
              onPointerUp={(event) => {
                if (drag.current?.id !== event.pointerId) return;
                drag.current = null;
                event.currentTarget.releasePointerCapture(event.pointerId);
              }}
              onPointerCancel={() => {
                drag.current = null;
              }}
              onLostPointerCapture={() => {
                drag.current = null;
              }}
            >
              <BoxFaces title={title} covers={covers} />
            </button>
          </div>
          <div className={styles.zoomControls} role="toolbar" aria-label={hebrew ? "הגדלת הקופסה" : "Box zoom"}>
            <button
              type="button"
              aria-label={hebrew ? "הקטנה" : "Zoom out"}
              disabled={zoom === 50}
              onClick={() => changeZoom(-25)}
            >
              <ZoomOut size={18} />
            </button>
            <output aria-live="polite" aria-label={hebrew ? "רמת הגדלה" : "Zoom level"}>
              {zoom}%
            </output>
            <button
              type="button"
              aria-label={hebrew ? "הגדלה" : "Zoom in"}
              disabled={zoom === 200}
              onClick={() => changeZoom(25)}
            >
              <ZoomIn size={18} />
            </button>
            <button type="button" onClick={() => setZoom(100)} disabled={zoom === 100}>
              {hebrew ? "איפוס הגדלה" : "Reset zoom"}
            </button>
          </div>
          <Dialog.Close className={styles.close} aria-label={hebrew ? "סגירת תצוגת הקופסה" : "Close game box viewer"}>
            <X size={20} />
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function BoxFaces({ title, covers }: Pick<GameBoxProps, "title" | "covers">) {
  return (
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
