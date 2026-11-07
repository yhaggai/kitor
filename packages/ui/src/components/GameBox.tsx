import { RotateCcw, ZoomIn, ZoomOut } from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "../i18n";
import type { GameSummary } from "../types";
import { type GameBoxLanguage, GameCover } from "./GameCover";

/** Interactive bilingual box viewer used in the inspection modal. */
export function GameBox({ game }: { game: GameSummary }) {
  const { t } = useTranslation();
  const [language, setLanguage] = useState<GameBoxLanguage>("he");
  const [rotation, setRotation] = useState(163);
  const [zoom, setZoom] = useState(100);
  const drag = useRef<{ id: number; x: number; rotation: number } | null>(null);
  const changeZoom = (delta: number) =>
    setZoom((value) => Math.min(150, Math.max(75, value + delta)));
  return (
    <div className="kt-box-viewer">
      <button
        type="button"
        className="kt-box-rotate"
        aria-label={t("Rotate {0} game box", [game.title])}
        aria-describedby={`box-help-${game.id}`}
        onKeyDown={(event) => {
          if (["ArrowLeft", "ArrowRight", "+", "=", "-", "0"].includes(event.key)) {
            event.preventDefault();
            if (event.key === "ArrowLeft" || event.key === "ArrowRight")
              setRotation((value) => value + (event.key === "ArrowLeft" ? -15 : 15));
            else if (event.key === "0") setZoom(100);
            else changeZoom(event.key === "-" ? -25 : 25);
          }
        }}
        onPointerDown={(event) => {
          if (!event.isPrimary || event.button !== 0) return;
          event.currentTarget.focus();
          event.currentTarget.setPointerCapture(event.pointerId);
          drag.current = { id: event.pointerId, x: event.clientX, rotation };
        }}
        onPointerMove={(event) => {
          if (drag.current?.id === event.pointerId)
            setRotation(drag.current.rotation + (event.clientX - drag.current.x) * 0.5);
        }}
        onPointerUp={(event) => {
          if (drag.current?.id === event.pointerId) {
            drag.current = null;
            event.currentTarget.releasePointerCapture(event.pointerId);
          }
        }}
        onPointerCancel={() => {
          drag.current = null;
        }}
        onLostPointerCapture={() => {
          drag.current = null;
        }}
      >
        <GameCover game={game} language={language} rotation={rotation} zoom={zoom} />
      </button>
      <fieldset className="kt-box-languages" aria-label={t("Box cover language")}>
        {(["he", "en"] as const).map((side) => (
          <button
            key={side}
            type="button"
            aria-pressed={language === side}
            lang={side}
            onClick={() => {
              setLanguage(side);
              setRotation(side === "he" ? 163 : -17);
            }}
          >
            {side === "he" ? "עברית" : t("English")}
          </button>
        ))}
      </fieldset>
      <div className="kt-box-tools">
        <button
          type="button"
          aria-label={t("Zoom out")}
          disabled={zoom === 75}
          onClick={() => changeZoom(-25)}
        >
          <ZoomOut size={17} />
        </button>
        <output aria-label={t("Box zoom level")}>{zoom}%</output>
        <button
          type="button"
          aria-label={t("Zoom in")}
          disabled={zoom === 150}
          onClick={() => changeZoom(25)}
        >
          <ZoomIn size={17} />
        </button>
        <button
          type="button"
          aria-label={t("Reset box view")}
          onClick={() => {
            setZoom(100);
            setRotation(language === "he" ? 163 : -17);
          }}
        >
          <RotateCcw size={16} />
        </button>
      </div>
      <p className="kt-box-help" id={`box-help-${game.id}`}>
        {t("Drag to rotate · Arrow keys to turn · + / − to zoom")}{" "}
      </p>
    </div>
  );
}
