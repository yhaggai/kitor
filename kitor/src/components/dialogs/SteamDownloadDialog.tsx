import { useTranslation } from "@kitor/ui";
import { useEffect, useState } from "react";
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
import { useGogInstallFlow } from "@/store/gogInstallFlow";
import { useNativeSteam } from "@/store/nativeSteam";
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
  const {
    busy,
    error: downloadError,
    start,
    appId: activeApp,
    downloadedBytes,
    totalBytes,
  } = useSteamDownload();
  const error = activeApp === appId ? downloadError : null;
  const [installRoot, setInstallRoot] = useState("");
  const { connected: direct, refresh } = useNativeSteam();
  const [connecting, setConnecting] = useState(false);
  useEffect(() => {
    if (open) void refresh();
  }, [open, refresh]);
  const collection = capabilities.games.length > 1;
  const nativeOwlboy = appId === 115800 && gameId === "owlboy";
  const guided = nativeOwlboy || (appId === 32460 && gameId === "monkey2-se");
  async function next() {
    if (useSteamDownload.getState().busy || connecting) return;
    setConnecting(true);
    onOpenChange(false);
    const ready = await useNativeSteam.getState().ensureConnected();
    setConnecting(false);
    if (!ready) {
      onOpenChange(true);
      return;
    }
    if (guided) {
      onOpenChange(false);
      useGogInstallFlow.getState().show({ steamAppId: appId, gameId, installRoot });
      return;
    }
    onOpenChange(true);
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
                : nativeOwlboy
                  ? t("Kitor downloads the native macOS edition of Owlboy from Steam.")
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
            {downloadedBytes !== null && (
              <output className="block text-sm" dir="ltr">
                {(downloadedBytes / 1_000_000).toFixed(1)} /{" "}
                {((totalBytes ?? 0) / 1_000_000).toFixed(1)} MB
              </output>
            )}
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
              {guided
                ? t("Kitor will install the game and its Hebrew patch.")
                : collection
                  ? t("Click Next to download the collection and detect its games.")
                  : t(
                      "Click Next to download and prepare the game. It will appear automatically in your library, ready to play. You can install the Hebrew patch separately."
                    )}
            </p>
            <p className="text-sm text-muted-foreground">
              {t(
                direct
                  ? "Direct downloads are connected. Steam will verify access to this game."
                  : "Next, connect Steam by scanning a QR code in Kitor. Your account must own this game."
              )}
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
            <Button disabled={!installRoot || connecting} onClick={() => void next()}>
              {error ? t("Try again") : t("Next")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
