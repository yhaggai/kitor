import { ChevronLeft, ChevronRight, Expand } from "lucide-react";
import { type KeyboardEvent, useState } from "react";
import { useTranslation } from "../i18n";
import type { GameSummary } from "../types";
import { Button } from "./Button";
import { FlowDialog } from "./FlowDialog";

export function GameGallery({
  title,
  screenshots = [],
}: Pick<GameSummary, "title" | "screenshots">) {
  const { t } = useTranslation();
  const [selected, setSelected] = useState(0);
  const [open, setOpen] = useState(false);
  const [failed, setFailed] = useState<string[]>([]);
  const images = screenshots.filter((image) => !failed.includes(image.src));
  const index = Math.min(selected, images.length - 1);
  const image = images[index];
  if (!image) return null;
  const move = (offset: number) => setSelected((index + offset + images.length) % images.length);
  const keyboard = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      event.preventDefault();
      move(event.key === "ArrowLeft" ? -1 : 1);
    }
  };
  const picture = (expanded = false) => (
    <img
      key={image.src}
      src={image.src}
      alt={image.alt || t("{0} — screenshot {1}", [title, index + 1])}
      className={expanded ? "kt-gallery-expanded" : "kt-gallery-image"}
      loading={expanded ? "eager" : "lazy"}
      onError={() => setFailed((previous) => [...previous, image.src])}
    />
  );
  const controls = (
    <div className="kt-gallery-controls" dir="ltr">
      <Button
        variant="secondary"
        aria-label={t("Previous screenshot")}
        disabled={images.length < 2}
        onClick={() => move(-1)}
      >
        <ChevronLeft size={18} />
      </Button>
      <span aria-live="polite" aria-atomic="true">
        {index + 1} / {images.length}
      </span>
      <Button
        variant="secondary"
        aria-label={t("Next screenshot")}
        disabled={images.length < 2}
        onClick={() => move(1)}
      >
        <ChevronRight size={18} />
      </Button>
    </div>
  );
  return (
    <section
      className="kt-gallery"
      aria-label={t("{0} screenshots", [title])}
      aria-roledescription="carousel"
      onKeyDown={keyboard}
    >
      <div className="kt-gallery-heading">
        <h2>{t("Gallery")}</h2>
        {controls}
      </div>
      <button
        type="button"
        className="kt-gallery-preview"
        aria-label={t("Enlarge screenshot {0}", [index + 1])}
        aria-haspopup="dialog"
        onClick={() => setOpen(true)}
      >
        {picture()}
        <span className="kt-box-expand">
          <Expand size={18} />
        </span>
      </button>
      {images.length > 1 && (
        <fieldset className="kt-gallery-thumbnails" aria-label={t("Choose screenshot")}>
          {images.map((item, position) => (
            <button
              type="button"
              key={item.src}
              aria-label={t("Show screenshot {0}", [position + 1])}
              aria-pressed={position === index}
              onClick={() => setSelected(position)}
            >
              <img src={item.src} alt="" loading="lazy" />
            </button>
          ))}
        </fieldset>
      )}
      {open && (
        <FlowDialog
          title={t("{0} · Gallery", [title])}
          onClose={() => setOpen(false)}
          className="kt-gallery-dialog"
        >
          {picture(true)}
          {controls}
        </FlowDialog>
      )}
    </section>
  );
}
