import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { SteamDownloadDialog } from "@/components/dialogs/SteamDownloadDialog";
import { Button } from "@/components/ui/button";
import { useSteamStore } from "@/store/steam";
import { useSteamDownload } from "@/store/steamDownload";
import { useAppStore } from "@/store/store";

type StoreInstallButtonsProps = {
  scummvmId: string;
  steamAppId?: number | null;
  gogId?: number | null;
};

export function StoreInstallButtons({ steamAppId, gogId, scummvmId }: StoreInstallButtonsProps) {
  const download = useSteamDownload();
  const [isMac, setIsMac] = useState<boolean | null>(null);
  const [downloadOpen, setDownloadOpen] = useState(false);
  useEffect(() => {
    void invoke<string>("get_os")
      .then((os) => setIsMac(os === "osx"))
      .catch(() => setIsMac(false));
  }, []);
  const supported =
    (steamAppId === 494740 && scummvmId === "pq1") ||
    (steamAppId === 32460 && scummvmId === "monkey2-se") ||
    (steamAppId === 10110 && ["sq1", "sq1sci", "sq2", "sq3", "sq5"].includes(scummvmId));
  const canDownload = isMac && supported;
  const steam = useSteamStore((state) => state.status);
  const ownsGogGame = useAppStore(
    (state) => state.gogAuth.isAuthenticated && state.gogGames.some((game) => game.id === gogId)
  );
  const ownsSteamGame = steamAppId != null && steam.owned_app_ids?.includes(steamAppId);
  const [opening, setOpening] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function openClient(store: "Steam" | "GOG") {
    setOpening(true);
    setError(null);
    try {
      if (store === "Steam") {
        await invoke("open_steam_client", { appId: steamAppId });
        if (!useSteamStore.getState().busy) void useSteamStore.getState().run("get_steam_status");
      } else {
        await invoke("open_gog_client", { gameId: gogId });
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setOpening(false);
    }
  }

  if (!steamAppId && !gogId) return null;

  return (
    <div className="w-full space-y-2">
      <div className="flex flex-wrap gap-2">
        {steamAppId && (
          <Button
            size="sm"
            variant="steam"
            className="h-12 flex-1 rounded-xl px-3 font-semibold tracking-wide transition-[filter,box-shadow]"
            disabled={opening || (supported && isMac === null)}
            title={
              canDownload
                ? "Download this game for your Mac"
                : ownsSteamGame
                  ? "Open this owned game in your Steam library"
                  : "Open this game’s store page in Steam"
            }
            onClick={() => {
              if (canDownload) {
                if (!download.busy) useSteamDownload.setState({ error: null });
                setDownloadOpen(true);
              } else {
                void openClient("Steam");
              }
            }}
          >
            <img
              src="/store-logos/steam.png"
              alt=""
              aria-hidden="true"
              className="h-8 w-8 shrink-0 rounded-md object-contain"
            />
            {canDownload && download.busy && download.appId === steamAppId
              ? "Downloading…"
              : "Steam"}
          </Button>
        )}
        {gogId && (
          <Button
            size="sm"
            variant="gog"
            className="h-12 flex-1 rounded-xl px-3 font-semibold tracking-wide transition-[filter,box-shadow]"
            disabled={opening}
            title={
              ownsGogGame
                ? "You own this game on GOG. Open in GOG Galaxy to install."
                : "Open this game in GOG Galaxy"
            }
            onClick={() => openClient("GOG")}
          >
            <img
              src="/store-logos/gog.png"
              alt=""
              aria-hidden="true"
              className="h-8 w-8 shrink-0 rounded-md object-cover"
            />
            {ownsGogGame ? "GOG · Owned" : "GOG"}
          </Button>
        )}
      </div>
      {canDownload && steamAppId && (
        <SteamDownloadDialog
          open={downloadOpen}
          onOpenChange={setDownloadOpen}
          appId={steamAppId}
          gameId={scummvmId}
        />
      )}
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
    </div>
  );
}
