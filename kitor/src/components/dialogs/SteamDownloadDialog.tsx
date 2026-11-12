import { useTranslation } from "@kitor/ui";
import { useEffect, useState } from "react";
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
  const [installRoot, setInstallRoot] = useState("");
  const { connected: direct, refresh } = useNativeSteam();
  const [connecting, setConnecting] = useState(false);
  useEffect(() => {
    if (open) void refresh();
  }, [open, refresh]);
  const collection = capabilities.games.length > 1;
  async function next() {
    if (connecting) return;
    setConnecting(true);
    onOpenChange(false);
    const ready = await useNativeSteam.getState().ensureConnected();
    setConnecting(false);
    if (!ready) {
      onOpenChange(true);
      return;
    }
    useGogInstallFlow.getState().show({ steamAppId: appId, gameId, installRoot });
  }
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("Install game from Steam")}</DialogTitle>
          <DialogDescription>
            {t("Kitor downloads the Steam files for this game and adds it to your library.")}
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-3">
          <GameInstallFolder
            value={installRoot}
            onChange={setInstallRoot}
            folders={[collection ? `steam-${appId}` : `${gameId}-steam`]}
          />
          <p className="text-sm text-muted-foreground">
            {t("Kitor will install the game and its Hebrew patch.")}
          </p>
          <p className="text-sm text-muted-foreground">
            {t(
              direct
                ? "Direct downloads are connected. Steam will verify access to this game."
                : "Next, connect Steam by scanning a QR code in Kitor. Your account must own this game."
            )}
          </p>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("Cancel")}
          </Button>
          <Button disabled={!installRoot || connecting} onClick={() => void next()}>
            {t("Next")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
