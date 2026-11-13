import { ChevronLeft, ChevronRight } from "lucide-react";
import {
  type ComponentType,
  type Ref,
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
} from "react";
import type { GameCategory } from "../categories";
import { GameTile } from "../components/GameTile";
import { useTranslation } from "../i18n";
import {
  type AnimatedIconHandle,
  BabyIcon,
  BookOpenIcon,
  Gamepad2Icon,
  GhostIcon,
  PuzzleIcon,
  RocketIcon,
  SearchIcon,
  SmileIcon,
  SparklesIcon,
  TimerIcon,
  TrophyIcon,
  WandSparklesIcon,
} from "../icons";
import type { GameSummary } from "../types";

type CategoryIcon = ComponentType<{
  ref?: Ref<AnimatedIconHandle>;
  size?: number;
}>;

const categoryIcons = new Map<string, CategoryIcon>([
  ["classic", TrophyIcon],
  ["detective", SearchIcon],
  ["thriller", TimerIcon],
  ["indie", SparklesIcon],
  ["comedy", SmileIcon],
  ["sci-fi", RocketIcon],
  ["fantasy", WandSparklesIcon],
  ["kids", BabyIcon],
  ["puzzle", PuzzleIcon],
  ["story", BookOpenIcon],
  ["horror", GhostIcon],
] satisfies [GameCategory, CategoryIcon][]);

export function CategoryRow({
  category,
  title,
  games,
  onOpen,
}: {
  category?: string;
  title: string;
  games: GameSummary[];
  onOpen: (game: GameSummary, element: HTMLButtonElement) => void;
}) {
  const { t } = useTranslation();
  const Icon = (category && categoryIcons.get(category)) || Gamepad2Icon;
  const iconRef = useRef<AnimatedIconHandle>(null);
  const headingId = useId();
  const trackId = useId();
  const trackRef = useRef<HTMLDivElement>(null);
  const [edges, setEdges] = useState({ start: true, end: true });

  const updateEdges = useCallback((track: HTMLDivElement) => {
    const offset = Math.abs(track.scrollLeft);
    const start = offset < 1;
    const end = track.scrollWidth - track.clientWidth - offset < 1;
    setEdges((previous) =>
      previous.start === start && previous.end === end ? previous : { start, end }
    );
  }, []);

  const count = games.length;
  useEffect(() => {
    const track = trackRef.current;
    if (!track || !count) return;
    const observer = new ResizeObserver(() => updateEdges(track));
    observer.observe(track);
    updateEdges(track);
    return () => observer.disconnect();
  }, [count, updateEdges]);

  function page(direction: number) {
    const track = trackRef.current;
    const tile = track?.firstElementChild;
    if (!track || !tile) return;
    const styles = getComputedStyle(track);
    const boxesPerPage = Number(styles.getPropertyValue("--kt-row-columns"));
    const step = tile.getBoundingClientRect().width + parseFloat(styles.columnGap);
    const firstBox = Math.round(Math.abs(track.scrollLeft) / step);
    const targetBox = Math.max(0, firstBox + direction * boxesPerPage);
    track.scrollTo({ left: targetBox * step * (styles.direction === "rtl" ? -1 : 1) });
  }

  return (
    <section className="kt-category-row" aria-labelledby={headingId}>
      <div className="kt-category-header">
        {/* Hover only plays decorative icon motion; the heading itself is not a control. */}
        {/* biome-ignore lint/a11y/noStaticElementInteractions: Decorative icon animation on title hover. */}
        <div
          className="kt-category-title"
          onMouseEnter={() => iconRef.current?.startAnimation()}
          onMouseLeave={() => iconRef.current?.stopAnimation()}
        >
          <Icon ref={iconRef} size={22} />
          <h2 id={headingId}>{title}</h2>
        </div>
      </div>
      <div className="kt-category-shelf">
        {!edges.start && (
          <button
            type="button"
            className="kt-category-arrow kt-category-arrow--previous"
            aria-label={t("Previous games")}
            aria-controls={trackId}
            onClick={() => page(-1)}
          >
            <ChevronLeft size={22} aria-hidden="true" />
          </button>
        )}
        <div
          id={trackId}
          className="kt-category-track"
          ref={trackRef}
          onScroll={(event) => updateEdges(event.currentTarget)}
        >
          {games.map((game) => (
            <GameTile key={game.id} game={game} onOpen={onOpen} />
          ))}
        </div>
        {!edges.end && (
          <button
            type="button"
            className="kt-category-arrow kt-category-arrow--next"
            aria-label={t("Next games")}
            aria-controls={trackId}
            onClick={() => page(1)}
          >
            <ChevronRight size={22} aria-hidden="true" />
          </button>
        )}
      </div>
    </section>
  );
}
