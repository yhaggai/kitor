import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { GogDownloadDialog } from "@/components/dialogs/GogDownloadDialog";
import {
  type SteamDownloadCapabilities,
  SteamDownloadDialog,
} from "@/components/dialogs/SteamDownloadDialog";
import { Button } from "@/components/ui/button";
import { useGogDownload } from "@/store/gogDownload";
import { useSteamStore } from "@/store/steam";
import { useSteamDownload } from "@/store/steamDownload";
import { useAppStore } from "@/store/store";

type StoreInstallButtonsProps = {
  scummvmId: string;
  steamAppId?: number | null;
  gogId?: number | null;
  installed?: boolean;
};

export function StoreInstallButtons({
  steamAppId,
  gogId,
  scummvmId,
  installed = false,
}: StoreInstallButtonsProps) {
  const download = useSteamDownload();
  const gogDownload = useGogDownload();
  const [gogSupported, setGogSupported] = useState<boolean | null>(null);
  const [gogOpen, setGogOpen] = useState(false);
  useEffect(() => {
    let active = true;
    setGogSupported(null);
    if (gogId) {
      void invoke<boolean>("get_gog_install_capabilities", { gameId: gogId, scummvmId })
        .then((supported) => {
          if (active) setGogSupported(supported);
        })
        .catch(() => {
          if (active) setGogSupported(false);
        });
    }
    return () => {
      active = false;
    };
  }, [gogId, scummvmId]);
  const [capabilities, setCapabilities] = useState<SteamDownloadCapabilities | null>(null);
  const [downloadOpen, setDownloadOpen] = useState(false);
  useEffect(() => {
    let active = true;
    setCapabilities(null);
    if (steamAppId) {
      void invoke<SteamDownloadCapabilities>("get_steam_download_capabilities", {
        appId: steamAppId,
        gameId: scummvmId,
      })
        .then((value) => {
          if (active) setCapabilities(value);
        })
        .catch(() => {
          if (active) setCapabilities({ supported: false, title: "", games: [] });
        });
    }
    return () => {
      active = false;
    };
  }, [steamAppId, scummvmId]);
  const canDownload = capabilities?.supported === true;
  const steam = useSteamStore((state) => state.status);
  const ownsGogGame = useAppStore(
    (state) => state.gogAuth.isAuthenticated && state.gogGames.some((game) => game.id === gogId)
  );
  const canInstallGog = gogSupported === true && ownsGogGame;
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
    <div className={installed ? "contents" : "w-full space-y-2"}>
      <div className={installed ? "hidden" : "flex flex-wrap gap-2"}>
        {steamAppId && (
          <Button
            size="sm"
            variant="steam"
            className="h-12 min-w-36 flex-1 rounded-xl px-3 font-semibold tracking-wide transition-[filter,box-shadow]"
            disabled={opening || capabilities === null}
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
              alt="Steam"
              className="h-8 w-8 shrink-0 rounded-md object-contain"
            />
            {canDownload && download.busy && download.appId === steamAppId
              ? "Downloading…"
              : canDownload
                ? "Install game"
                : "Steam"}
          </Button>
        )}
        {gogId && (
          <Button
            size="sm"
            variant="gog"
            className="h-12 min-w-36 flex-1 rounded-xl px-3 font-semibold tracking-wide transition-[filter,box-shadow]"
            disabled={opening || gogSupported === null}
            title={
              canInstallGog
                ? "Install this game for ScummVM on your Mac"
                : ownsGogGame
                  ? "You own this game on GOG. Open in GOG Galaxy to install."
                  : "Open this game in GOG Galaxy"
            }
            onClick={() => {
              if (canInstallGog) {
                if (!gogDownload.busy) {
                  useGogDownload.setState({ error: null, complete: false });
                }
                setGogOpen(true);
              } else {
                void openClient("GOG");
              }
            }}
          >
            <img
              src="/store-logos/gog.png"
              alt="GOG"
              className="h-8 w-8 shrink-0 rounded-md object-cover"
            />
            {canInstallGog
              ? gogDownload.busy && gogDownload.gameId === scummvmId
                ? "Installing…"
                : "Install game"
              : ownsGogGame
                ? "GOG · Owned"
                : "GOG"}
          </Button>
        )}
      </div>
      {canDownload && capabilities && steamAppId && (
        <SteamDownloadDialog
          open={downloadOpen}
          onOpenChange={setDownloadOpen}
          capabilities={capabilities}
          appId={steamAppId}
          gameId={scummvmId}
        />
      )}
      {canInstallGog && gogId && (
        <GogDownloadDialog
          open={gogOpen}
          onOpenChange={setGogOpen}
          gogId={gogId}
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
