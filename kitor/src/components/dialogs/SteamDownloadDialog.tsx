import { useTranslation } from "@kitor/ui";
import { useState } from "react";
import BugReportDialog from "@/components/BugReportDialog";
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
import { useSteamDownload } from "@/store/steamDownload";

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
  const { t } = useTranslation();
  const { busy, error: downloadError, start, appId: activeApp } = useSteamDownload();
  const error = activeApp === appId ? downloadError : null;
  const [installRoot, setInstallRoot] = useState("");
  const collection = capabilities.games.length > 1;
  async function next() {
    if (useSteamDownload.getState().busy) return;
    await start(appId, [gameId], installRoot);
    if (!useSteamDownload.getState().error) onOpenChange(false);
  }
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {busy
              ? t("Preparing Steam games")
              : collection
                ? t("Download {0}", [capabilities.title])
                : t("Install game from Steam")}
          </DialogTitle>
          <DialogDescription>
            {busy
              ? activeApp === appId
                ? t(
                    "Kitor is downloading or reusing the game files, verifying them, and detecting available games for your library."
                  )
                : t(
                    "Another Steam download is running. Wait for it to finish before starting this one."
                  )
              : collection
                ? t(
                    "Kitor downloads and extracts the full collection. Game detection identifies the games available to play and patch on your Mac."
                  )
                : t(
                    "This game is only available for Windows on Steam. Kitor can download that version and run it on your Mac with ScummVM."
                  )}
          </DialogDescription>
        </DialogHeader>
        {busy ? (
          <div className="space-y-3">
            <div
              role="progressbar"
              aria-label={t("Downloading and verifying game files")}
              aria-valuetext="In progress"
              className="h-3 overflow-hidden rounded-full bg-secondary"
            >
              <div className="steam-download-indicator h-full w-1/3 rounded-full bg-primary" />
            </div>
            <output className="block text-sm text-muted-foreground">
              {t(
                "Keep Kitor open. Hiding this dialog does not cancel Steam’s download. Reopen it from any Steam button in this collection."
              )}{" "}
            </output>
          </div>
        ) : (
          <div className="space-y-3">
            <GameInstallFolder
              value={installRoot}
              onChange={setInstallRoot}
              folders={[collection ? `steam-${appId}` : `${gameId}-steam`]}
            />
            <p className="text-sm text-muted-foreground">
              {collection
                ? t("Click Next to download the collection and detect its games.")
                : t(
                    "Click Next to download and prepare the game. It will appear automatically in your library, ready to play. You can install the Hebrew patch separately."
                  )}
            </p>
            <p className="text-sm text-muted-foreground">
              {t("Sign in to Steam with an account that owns")}{" "}
              {collection ? capabilities.title : t("the game")}
              {t(
                ". Steam may open its console. Hebrew patches are installed separately and must support your game edition."
              )}{" "}
            </p>
          </div>
        )}
        {error && (
          <div className="space-y-2">
            <p role="alert" className="text-sm text-destructive">
              {error}
            </p>
            <BugReportDialog game={gameId} details={error} />
          </div>
        )}
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {busy ? t("Hide") : t("Cancel")}
          </Button>
          {!busy && (
            <Button disabled={!installRoot} onClick={() => void next()}>
              {error ? t("Try again") : t("Next")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
