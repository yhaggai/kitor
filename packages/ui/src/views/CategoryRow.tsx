import { Tag } from "@kitor/registry";
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
import hotKitorEmoji from "../assets/hot-kitor.webp";
import type { GameCategory } from "../categories";
import { GameSort, type GameSortOrder, sortGamesByRelease } from "../components/GameSort";
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

const categoryIcons: Record<GameCategory, CategoryIcon> = {
  [Tag.Classic]: TrophyIcon,
  [Tag.Detective]: SearchIcon,
  [Tag.Thriller]: TimerIcon,
  [Tag.Indie]: SparklesIcon,
  [Tag.Comedy]: SmileIcon,
  [Tag.SciFi]: RocketIcon,
  [Tag.Fantasy]: WandSparklesIcon,
  [Tag.Kids]: BabyIcon,
  [Tag.Puzzle]: PuzzleIcon,
  [Tag.Story]: BookOpenIcon,
  [Tag.Horror]: GhostIcon,
  [Tag.HotKitor]: SparklesIcon,
};

export function CategoryRow({
  category,
  title,
  logo,
  logoOnDark,
  games,
  onOpen,
  sortOrder,
  onSortChange,
}: {
  category?: GameCategory;
  title: string;
  logo?: string;
  logoOnDark?: boolean;
  games: GameSummary[];
  onOpen: (game: GameSummary, element: HTMLButtonElement) => void;
  sortOrder: GameSortOrder;
  onSortChange: (order: GameSortOrder) => void;
}) {
  const { t } = useTranslation();
  const isHotKitor = category === Tag.HotKitor;
  const orderedGames =
    sortOrder === "default" && isHotKitor
      ? [...games].sort((a, b) => (b.addedOn ?? "").localeCompare(a.addedOn ?? ""))
      : sortGamesByRelease(games, sortOrder);
  const Icon = (category && categoryIcons[category]) || Gamepad2Icon;
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
          {logo ? (
            <img
              className={logoOnDark ? "kt-studio-logo kt-studio-logo--dark" : "kt-studio-logo"}
              src={logo}
              alt=""
              width={72}
              height={30}
            />
          ) : isHotKitor ? (
            <img className="kt-category-emoji" src={hotKitorEmoji} alt="" width={30} height={30} />
          ) : (
            <Icon ref={iconRef} size={22} />
          )}
          <h2 id={headingId} dir="auto">
            {title}
          </h2>
        </div>
        <GameSort
          scope={title}
          order={sortOrder}
          onChange={(order) => {
            onSortChange(order);
            trackRef.current?.scrollTo({ left: 0, behavior: "instant" });
          }}
        />
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
          {orderedGames.map((game) => (
            <GameTile key={game.id} game={game} onOpen={onOpen} showBeta={game.translationBeta} />
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
