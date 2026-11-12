import { gogIcon, kitorIcon, ManageMenu, steamIcon, useTranslation } from "@kitor/ui";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ChevronDown, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import AddGameDialog from "@/components/AddGameDialog";
import {
  type SteamDownloadCapabilities,
  SteamDownloadDialog,
} from "@/components/dialogs/SteamDownloadDialog";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { gogStoreUrl } from "@/lib/gameAction";
import { useGogInstallFlow } from "@/store/gogInstallFlow";
import { useNativeSteam } from "@/store/nativeSteam";
import { useSteamStore } from "@/store/steam";
import { useSteamDownload } from "@/store/steamDownload";
import { type Game, useAppStore } from "@/store/store";

type CatalogGameActionsProps = {
  scummvmId: string;
  steamAppId?: number | null;
  gogId?: number | null;
  gogInstallerPath?: string | null;
  addToCartUrl?: string | null;
};

export function CatalogGameActions({
  scummvmId,
  steamAppId,
  gogId,
  gogInstallerPath,
  addToCartUrl,
}: CatalogGameActionsProps) {
  const { t } = useTranslation();
  const download = useSteamDownload();
  const steam = useSteamStore((state) => state.status);
  const gogGames = useAppStore((state) => state.gogGames);
  const gogAuthenticated = useAppStore((state) => state.gogAuth.isAuthenticated);
  const ownsSteam = steamAppId != null && (steam.owned_app_ids?.includes(steamAppId) ?? false);
  const ownsGog = gogId != null && gogAuthenticated && gogGames.some((game) => game.id === gogId);
  const [capabilities, setCapabilities] = useState<SteamDownloadCapabilities | null>(null);
  const [downloadOpen, setDownloadOpen] = useState(false);
  const [addOpen, setAddOpen] = useState(false);
  const [installSource, setInstallSource] = useState<"gog" | "steam" | null>(null);
  const [waitingForClient, setWaitingForClient] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [clientError, setClientError] = useState<string | null>(null);
  const [opening, setOpening] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

  const steamBusy = download.busy && download.appId === steamAppId;

  async function chooseStore(provider: "gog" | "steam") {
    setError(null);
    if (provider === "gog" && gogId) {
      useGogInstallFlow.getState().show({
        gogId,
        gameId: scummvmId,
        purchaseUrl: addToCartUrl || gogStoreUrl(gogInstallerPath),
      });
      return;
    }
    if (provider !== "steam" || !steamAppId) return;
    if (capabilities?.supported) {
      if (!download.busy) useSteamDownload.setState({ error: null });
      setDownloadOpen(true);
      return;
    }
    setOpening(true);
    try {
      await invoke("open_steam_client", { appId: steamAppId });
    } catch (err) {
      setError(String(err));
    } finally {
      setOpening(false);
    }
  }

  async function refreshGame() {
    if (refreshing) return;
    setRefreshing(true);
    setError(null);
    try {
      await invoke("scan_installed_games");
      const games = await invoke<Game[]>("get_games_with_patches");
      useAppStore.getState().setGames(games);
    } catch (err) {
      setError(String(err));
    } finally {
      setRefreshing(false);
    }
  }

  const [selectedStore, setSelectedStore] = useState<"gog" | "steam" | null>(null);
  const primaryStore =
    selectedStore ?? (gogId && (ownsGog || !ownsSteam || !steamAppId) ? "gog" : "steam");
  const waitingForSteam = primaryStore === "steam" && capabilities === null;
  const installLabel = (provider: "gog" | "steam") =>
    t(
      provider === "steam"
        ? steam.owned_app_ids === null
          ? "Play on Steam"
          : ownsSteam
            ? "Install from Steam"
            : "Buy on Steam"
        : !gogAuthenticated
          ? "Play on GOG"
          : ownsGog
            ? "Install from GOG"
            : "Buy on GOG"
    );

  async function selectStore(provider: "gog" | "steam") {
    setError(null);
    if (provider === "steam") {
      if (!steamAppId) return;
      setOpening(true);
      try {
        if (!(await useNativeSteam.getState().ensureConnected())) return;
        const library = useSteamStore.getState();
        if (library.status.owned_app_ids === null) {
          if (!(await library.run("refresh_steam_library"))) {
            setError(useSteamStore.getState().error || "Library could not be loaded");
            return;
          }
        }
        const owned = useSteamStore.getState().status.owned_app_ids?.includes(steamAppId);
        if (owned) setInstallSource(provider);
        else await openUrl(`https://store.steampowered.com/app/${steamAppId}/`);
      } catch (err) {
        setError(String(err));
      } finally {
        setOpening(false);
      }
      return;
    }
    if (provider === "gog" && !ownsGog) {
      if (!gogAuthenticated) {
        await chooseStore(provider);
      } else {
        const url =
          addToCartUrl ||
          gogStoreUrl(gogInstallerPath) ||
          `https://www.gog.com/checkout/manual/${gogId}`;
        try {
          await openUrl(url);
        } catch (err) {
          setError(String(err));
        }
      }
      return;
    }
    setInstallSource(provider);
  }

  return (
    <div className="kt-game-actions">
      {gogId || steamAppId ? (
        <div className="kt-install-split">
          <button
            type="button"
            dir="auto"
            className="kt-game-action kt-game-action--solid inline-flex items-center justify-center gap-2"
            onClick={() => void selectStore(primaryStore)}
            disabled={opening || waitingForSteam}
            aria-busy={waitingForSteam || undefined}
          >
            <img
              src={primaryStore === "steam" ? steamIcon : gogIcon}
              alt=""
              className="h-6 w-6 shrink-0 object-contain"
            />
            {steamBusy && primaryStore === "steam" ? t("Downloading…") : installLabel(primaryStore)}
          </button>
          {
            <DropdownMenu.Root>
              <DropdownMenu.Trigger asChild>
                <button
                  type="button"
                  className="kt-game-action kt-install-arrow"
                  aria-label={t("Choose an install source")}
                  disabled={opening}
                >
                  <ChevronDown size={18} aria-hidden="true" />
                </button>
              </DropdownMenu.Trigger>
              <DropdownMenu.Portal>
                <DropdownMenu.Content
                  align="end"
                  sideOffset={6}
                  className="z-50 min-w-44 rounded-md border bg-background p-1 shadow-lg"
                >
                  {(["steam", "gog"] as const).map((provider) => (
                    <DropdownMenu.Item
                      key={provider}
                      dir="auto"
                      className="flex items-center gap-2 cursor-pointer rounded-sm px-3 py-2 text-sm text-foreground outline-none data-[highlighted]:bg-accent data-[disabled]:opacity-50"
                      disabled={provider === "steam" ? !steamAppId : !gogId}
                      onSelect={() => {
                        setError(null);
                        setSelectedStore(provider);
                        void selectStore(provider);
                      }}
                    >
                      <img
                        src={provider === "steam" ? steamIcon : gogIcon}
                        alt=""
                        className="h-6 w-6 shrink-0 object-contain"
                      />
                      {installLabel(provider)}
                    </DropdownMenu.Item>
                  ))}
                  <DropdownMenu.Separator className="my-1 h-px bg-border" />
                  <DropdownMenu.Item
                    className="flex items-center gap-2 cursor-pointer rounded-sm px-3 py-2 text-sm text-foreground outline-none data-[highlighted]:bg-accent data-[disabled]:opacity-50"
                    disabled={refreshing}
                    onSelect={() => void refreshGame()}
                  >
                    <RefreshCw className={refreshing ? "h-4 w-4 animate-spin" : "h-4 w-4"} />
                    {t(refreshing ? "Refreshing…" : "Refresh game")}
                  </DropdownMenu.Item>
                </DropdownMenu.Content>
              </DropdownMenu.Portal>
            </DropdownMenu.Root>
          }
        </div>
      ) : addToCartUrl ? (
        <button
          type="button"
          className="kt-game-action kt-game-action--solid"
          onClick={() => void openUrl(addToCartUrl).catch((err) => setError(String(err)))}
        >
          {t("Get the game")}
        </button>
      ) : null}
      {!gogId && !steamAppId && (
        <ManageMenu label={t("Refresh game")}>
          <Button variant="outline" disabled={refreshing} onClick={() => void refreshGame()}>
            <RefreshCw className={refreshing ? "h-4 w-4 animate-spin" : "h-4 w-4"} />
            {t(refreshing ? "Refreshing…" : "Refresh game")}
          </Button>
        </ManageMenu>
      )}
      {!ownsGog && !ownsSteam && (
        <button
          type="button"
          className="kt-game-action kt-game-action--own"
          onClick={() => setAddOpen(true)}
        >
          {t("I already have the game")}
        </button>
      )}
      {error && (
        <p role="alert" className="kt-game-action-status">
          {error}
        </p>
      )}
      <Dialog
        open={installSource !== null}
        onOpenChange={(open) => {
          if (!open && !opening) setInstallSource(null);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("Choose how to install")}</DialogTitle>
            <DialogDescription>
              {t("Download with the store client or directly through Kitor.")}
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-3">
            <Button
              variant="default"
              disabled={opening || (installSource === "steam" && !capabilities?.supported)}
              onClick={() => {
                const provider = installSource;
                setInstallSource(null);
                if (provider) void chooseStore(provider);
              }}
            >
              <img src={kitorIcon} alt="" className="h-6 w-12 object-contain" />
              {t("Download with Kitor")}
            </Button>
            <Button
              variant="outline"
              disabled={opening}
              onClick={async () => {
                setOpening(true);
                setError(null);
                try {
                  if (installSource === "steam") {
                    await invoke("open_steam_client", { appId: steamAppId });
                    setClientError(null);
                    setWaitingForClient(true);
                  } else await invoke("open_gog_client", { gameId: gogId });
                  setInstallSource(null);
                } catch (err) {
                  setError(String(err));
                } finally {
                  setOpening(false);
                }
              }}
            >
              <img
                src={installSource === "steam" ? steamIcon : gogIcon}
                alt=""
                className="h-6 w-6 object-contain"
              />
              {t(installSource === "steam" ? "Use Steam client" : "Use GOG Galaxy")}
            </Button>
            {installSource === "steam" && capabilities?.supported === false && (
              <p className="text-sm text-muted-foreground">
                {t("Direct download is not available for this game yet.")}
              </p>
            )}
            {error && (
              <p role="alert" className="text-sm text-destructive">
                {t(error)}
              </p>
            )}
          </div>
        </DialogContent>
      </Dialog>
      <Dialog
        open={waitingForClient}
        onOpenChange={(open) => {
          if (!scanning) setWaitingForClient(open);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("Install the game through Steam")}</DialogTitle>
            <DialogDescription>
              {t(
                "After installing the game in Steam, return to this window and click “I installed the game”."
              )}
            </DialogDescription>
          </DialogHeader>
          <Button
            disabled={scanning}
            onClick={async () => {
              setScanning(true);
              setClientError(null);
              try {
                await invoke("scan_installed_games");
                const games = await invoke<Game[]>("get_games_with_patches");
                const installed = games.find((game) => game.scummvm_id === scummvmId);
                if (installed) {
                  setWaitingForClient(false);
                  if (steamAppId)
                    useGogInstallFlow.getState().show({
                      steamAppId,
                      gameId: scummvmId,
                      installedGameId: installed.id,
                    });
                  useAppStore.getState().setGames(games);
                } else {
                  setClientError(
                    "The game was not found yet. Finish installing it in Steam, then try again."
                  );
                }
              } catch (err) {
                setClientError(String(err));
              } finally {
                setScanning(false);
              }
            }}
          >
            {t(scanning ? "Looking for the installed game…" : "I installed the game")}
          </Button>
          {clientError && (
            <p role="alert" className="text-sm text-destructive">
              {t(clientError)}
            </p>
          )}
        </DialogContent>
      </Dialog>
      {capabilities?.supported && steamAppId && (
        <SteamDownloadDialog
          open={downloadOpen}
          onOpenChange={setDownloadOpen}
          capabilities={capabilities}
          appId={steamAppId}
          gameId={scummvmId}
        />
      )}
      <AddGameDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        onAdded={async () => {
          useAppStore.getState().setGames(await invoke<Game[]>("get_games_with_patches"));
        }}
      />
    </div>
  );
}
