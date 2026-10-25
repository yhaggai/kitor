import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { useRef } from "react";
import { GameBox, type GameBoxLanguage, type GameBoxCover } from "@/components/GameBox";
import gameBoxCovers from "@/data/game-box-covers.json";
import { getCoverSrc } from "@/lib/utils";

type GameDetailsBoxProps = {
  gameId: string;
  title: string;
  coverUrl?: string;
  description?: string;
  isPatched?: boolean;
};

/** Keep details and cover flipping separate from menu and play actions. */
export function GameDetailsBox({
  gameId,
  title,
  coverUrl,
  description,
  isPatched = false,
}: GameDetailsBoxProps) {
  const defaultLanguage = isPatched ? "he" : "en";
  const covers = (gameBoxCovers as Record<string, Record<GameBoxLanguage, GameBoxCover>>)[
    gameId
  ] ?? {
    en: { src: getCoverSrc(coverUrl) ?? "" },
    he: { src: "" },
  };
  const box = (location: string) => (
    <GameBox
      key={`${gameId}:${defaultLanguage}:${location}`}
      title={title}
      covers={covers}
      defaultLanguage={defaultLanguage}
      showLanguageControls={false}
    />
  );
  const trigger = useRef<HTMLButtonElement>(null);
  const content = useRef<HTMLDivElement>(null);

  const animateOpen = () => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const source = trigger.current?.getBoundingClientRect();
    const target = content.current?.getBoundingClientRect();
    if (!source || !target) return;
    content.current?.animate(
      [
        {
          transform: `translate(${source.left - target.left}px, ${source.top - target.top}px) scale(${source.width / target.width}, ${source.height / target.height})`,
          opacity: 0.35,
          borderRadius: "12px",
        },
        { transform: "translate(0, 0) scale(1)", opacity: 1, borderRadius: "20px" },
      ],
      { duration: 380, easing: "cubic-bezier(0.22, 1, 0.36, 1)" }
    );
  };

  return (
    <div className="w-full min-w-0 pt-8">
      <Dialog.Root>
        {box("card")}
        <Dialog.Trigger asChild>
          <button
            ref={trigger}
            type="button"
            aria-label={`About ${title}`}
            className="w-full px-4 py-3 text-sm font-medium text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            Explore game
          </button>
        </Dialog.Trigger>
        <Dialog.Portal>
          <Dialog.Overlay className="game-details-overlay fixed inset-0 z-50 bg-black/70 backdrop-blur-sm" />
          <div className="pointer-events-none fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-8">
            <Dialog.Content
              ref={content}
              onOpenAutoFocus={animateOpen}
              className="game-details-content pointer-events-auto relative w-full max-w-3xl origin-top-left overflow-y-auto rounded-[20px] border bg-background text-foreground shadow-2xl max-h-[85dvh]"
            >
              <div className="grid sm:grid-cols-[minmax(0,2fr)_minmax(0,3fr)]">
                <div className="min-w-0 p-4 pt-12">{box("details")}</div>
                <div className="p-6 sm:p-8 sm:pt-12">
                  <p className="mb-3 text-xs font-medium uppercase tracking-widest text-muted-foreground">
                    About the game
                  </p>
                  <Dialog.Title
                    dir="auto"
                    className="mb-5 text-2xl font-semibold leading-tight text-start"
                  >
                    {title}
                  </Dialog.Title>
                  <Dialog.Description asChild>
                    <div className="space-y-4 text-sm leading-7 text-muted-foreground">
                      {(description?.trim() || "A description isn't available for this game yet.")
                        .split(/\n\s*\n/)
                        .map((paragraph, index) => (
                          <p
                            key={`${index}-${paragraph}`}
                            dir="auto"
                            className="whitespace-pre-line text-start"
                          >
                            {paragraph}
                          </p>
                        ))}
                    </div>
                  </Dialog.Description>
                </div>
              </div>
              <Dialog.Close
                aria-label="Close game description"
                className="absolute right-3 top-3 rounded-full border bg-background/90 p-2 text-foreground shadow-sm transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <X className="h-4 w-4" />
              </Dialog.Close>
            </Dialog.Content>
          </div>
        </Dialog.Portal>
      </Dialog.Root>
    </div>
  );
}
