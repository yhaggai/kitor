import { useState } from "react";
import { GameInstallFolder } from "@/components/dialogs/GameInstallFolder";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useGogDownload } from "@/store/gogDownload";
import { useAppStore } from "@/store/store";

export function GogDownloadDialog({
  open,
  onOpenChange,
  gogId,
  gameId,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  gogId: number;
  gameId: string;
}) {
  const download = useGogDownload();
  const ownsGame = useAppStore(
    (s) => s.gogAuth.isAuthenticated && s.gogGames.some((game) => game.id === gogId)
  );
  const [installRoot, setInstallRoot] = useState("");
  const active = download.gameId === gameId;
  const complete = active && download.complete;
  const error = active ? download.error : null;
  function done() {
    onOpenChange(false);
    useAppStore.getState().setActiveTab("library");
  }
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {complete
              ? "Game installed"
              : download.busy
                ? "Installing game"
                : "Install game from GOG"}
          </DialogTitle>
          <DialogDescription>
            {complete
              ? "Your game is in the library and ready to play through ScummVM. You can install the Hebrew patch separately whenever you want."
              : download.busy
                ? active
                  ? "Please wait while Kitor downloads and prepares your game."
                  : "Another GOG game is being installed. Please wait for it to finish."
                : "This installer is only supported on Windows. Kitor will download and extract the game files so you can play on your Mac through ScummVM."}
          </DialogDescription>
        </DialogHeader>
        {download.busy || complete ? (
          <div className="space-y-3" aria-live="polite">
            <div
              role="progressbar"
              aria-label="Game installation"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={active ? (download.percentage ?? undefined) : undefined}
              aria-valuetext={active ? download.step : "Waiting for another installation"}
              className="h-3 overflow-hidden rounded-full bg-secondary"
            >
              <div
                className={
                  active && download.percentage != null
                    ? "h-full bg-primary transition-all"
                    : "steam-download-indicator h-full w-1/3 rounded-full bg-primary"
                }
                style={
                  active && download.percentage != null
                    ? { width: `${download.percentage}%` }
                    : undefined
                }
              />
            </div>
            <p className="text-sm">
              {active ? download.step : "Installation in progress…"}
              {active && download.percentage != null ? ` ${download.percentage}%` : ""}
            </p>
            {!complete && (
              <p className="text-xs text-muted-foreground">
                Keep Kitor open until installation finishes. You can hide this window and reopen it
                from Install game.
              </p>
            )}
          </div>
        ) : (
          <div className="space-y-4">
            <GameInstallFolder
              value={installRoot}
              onChange={setInstallRoot}
              folders={[`${gameId}-gog`]}
            />
            <p className="text-sm text-muted-foreground">
              Click Next to download and prepare the game. It will appear automatically in your
              library, ready to play. Installing the Hebrew patch is optional and is a separate
              step.
            </p>
            {!ownsGame && (
              <p className="text-sm">
                Connect the GOG account that owns this game in Settings before continuing.
              </p>
            )}
          </div>
        )}
        {error && (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        )}
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {download.busy ? "Hide" : complete ? "Close" : "Cancel"}
          </Button>
          {complete ? (
            <Button onClick={done}>View library</Button>
          ) : (
            !download.busy && (
              <Button
                disabled={!ownsGame || !installRoot}
                onClick={() => void download.start(gogId, gameId, installRoot)}
              >
                {error ? "Try again" : "Next"}
              </Button>
            )
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
