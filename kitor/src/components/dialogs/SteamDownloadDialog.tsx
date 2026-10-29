import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useSteamDownload } from "@/store/steamDownload";
import { useAppStore } from "@/store/store";

export type SteamDownloadCapabilities = {
  supported: boolean;
  title: string;
  games: { id: string; title: string }[];
};

export function SteamDownloadDialog({
  open,
  onOpenChange,
  appId,
  gameId,
  capabilities,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  appId: number;
  gameId: string;
  capabilities: SteamDownloadCapabilities;
}) {
  const { busy, error, start, appId: activeApp } = useSteamDownload();
  const games = useAppStore((state) => state.games);
  const [selected, setSelected] = useState([gameId]);
  useEffect(() => {
    if (open) setSelected([gameId]);
  }, [open, gameId]);
  const collection = capabilities.games.length > 1;
  async function next() {
    if (useSteamDownload.getState().busy) return;
    await start(appId, [gameId, ...selected.filter((id) => id !== gameId)]);
    if (!useSteamDownload.getState().error) onOpenChange(false);
  }
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {busy
              ? "Preparing Steam games"
              : collection
                ? `Install from ${capabilities.title}`
                : "Download the Windows version?"}
          </DialogTitle>
          <DialogDescription>
            {busy
              ? activeApp === appId
                ? "Kitor is downloading or reusing the game files, verifying them, and adding your selected games to the library."
                : "Another Steam download is running. Wait for it to finish before starting this one."
              : collection
                ? "These games share one Steam collection. Kitor downloads it once and imports each selected game separately for ScummVM on your Mac."
                : "This game is only available for Windows on Steam. Kitor can download that version and run it on your Mac with ScummVM."}
          </DialogDescription>
        </DialogHeader>
        {busy ? (
          <div className="space-y-3">
            <div
              role="progressbar"
              aria-label="Downloading and verifying game files"
              aria-valuetext="In progress"
              className="h-3 overflow-hidden rounded-full bg-secondary"
            >
              <div className="steam-download-indicator h-full w-1/3 rounded-full bg-primary" />
            </div>
            <output className="block text-sm text-muted-foreground">
              Keep Kitor open. Hiding this dialog does not cancel Steam’s download. Reopen it from
              any Steam button in this collection.
            </output>
          </div>
        ) : (
          <div className="space-y-3">
            {collection && (
              <fieldset className="space-y-2">
                <legend className="mb-2 text-sm font-medium">Games to add</legend>
                {capabilities.games.map(({ id, title }) => {
                  const installed = games.some((game) => game.scummvm_id === id);
                  return (
                    <label key={id} className="flex items-center gap-2 text-sm">
                      <input
                        type="checkbox"
                        checked={selected.includes(id) || installed}
                        disabled={id === gameId || installed}
                        onChange={(event) =>
                          setSelected((current) =>
                            event.target.checked
                              ? [...current, id]
                              : current.filter((item) => item !== id)
                          )
                        }
                      />
                      {title}
                      {installed ? " — already in library" : ""}
                    </label>
                  );
                })}
              </fieldset>
            )}
            <p className="text-sm text-muted-foreground">
              Sign in to Steam with an account that owns{" "}
              {collection ? capabilities.title : "the game"}. Steam may open its console. Hebrew
              patches are installed separately and must support your game edition.
            </p>
          </div>
        )}
        {error && (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        )}
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {busy ? "Hide" : "Cancel"}
          </Button>
          {!busy && <Button onClick={() => void next()}>{error ? "Try again" : "Next"}</Button>}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
