import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import * as Tooltip from "@radix-ui/react-tooltip";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  CheckCircle,
  ChevronDown,
  Download,
  ExternalLink,
  FolderSearch,
  MoreVertical,
  Play,
} from "lucide-react";
import { useEffect, useState } from "react";
import AddGameDialog from "@/components/AddGameDialog";
import { GameDetailsBox } from "@/components/GameDetailsBox";
import { OpenGameFolderMenuItem } from "@/components/OpenGameFolderMenuItem";
import { PurchaseGuideDialog } from "@/components/PurchaseGuideDialog";
import { StoreInstallButtons } from "@/components/StoreInstallButtons";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import { usePatchBrowserState } from "@/hooks/usePatchBrowserState";
import { type Game, type Patch, type PatchWithStatus, useAppStore } from "@/store/store";

export default function PatchBrowser() {
  const { state, actions } = usePatchBrowserState();
  const { patches, loading, showAddDialog, selectedGuide } = state;

  const { games, setActiveTab, setHighlightedGameId, setGames } = useAppStore();
  const [isScanning, setIsScanning] = useState(false);
  const [scanMessage, setScanMessage] = useState("");
  const [scanWarnings, setScanWarnings] = useState<string[]>([]);

  const handleScanGames = async (automatic: boolean) => {
    if (isScanning) return;
    setIsScanning(true);
    setScanMessage("");
    setScanWarnings([]);
    try {
      if (automatic) {
        const result = await invoke<{
          added_count: number;
          locations: string[];
          warnings: string[];
        }>("scan_installed_games");
        setScanMessage(
          result.locations.length === 0
            ? "No Steam or GOG folders found. Try choosing a folder."
            : `Scanned ${result.locations.length} locations. Added ${result.added_count} games. Only recognized games are added; existing games are skipped.`
        );
        setScanWarnings(result.warnings);
      } else {
        const selected = await openDialog({
          directory: true,
          multiple: false,
          title: "Select folder to scan for games",
        });
        if (!selected) return;
        const added = await invoke<unknown[]>("scan_games", { scanPath: selected });
        setScanMessage(
          `Added ${added.length} games. Only recognized games are added; existing games are skipped.`
        );
      }
    } catch (error) {
      setScanMessage(`Scan failed: ${String(error)}`);
    } finally {
      try {
        setGames(await invoke<Game[]>("get_games_with_patches"));
      } catch (error) {
        setScanWarnings((warnings) => [...warnings, `Could not refresh games: ${String(error)}`]);
      } finally {
        setIsScanning(false);
      }
    }
  };

  useEffect(() => {
    async function loadPatches() {
      try {
        actions.setLoading(true);
        const allPatches = (await invoke("get_all_patches")) as Patch[];

        // Map status to patches
        const patchesWithStatus = allPatches.map((patch) => {
          const game = games.find((g) => g.scummvm_id === patch.scummvm_id);
          const isGameInstalled = !!game;
          const isPatchInstalled =
            game?.installed_patches?.some(
              (p) => p.language === patch.language && p.version === patch.version
            ) ?? false;

          return {
            ...patch,
            game_installed: isGameInstalled,
            patch_installed: isPatchInstalled,
            game_id: game?.id,
            cover_url: game?.cover_url || patch.cover_url,
          };
        });

        actions.setPatches(patchesWithStatus);
      } catch (error) {
        console.error("Failed to load patches:", error);
      } finally {
        actions.setLoading(false);
      }
    }

    loadPatches();
  }, [games, actions]);

  const selectedPatch = patches.find((patch) => patch.scummvm_id === selectedGuide?.scummvmId);

  const handleViewGame = (gameId: number) => {
    setActiveTab("library");
    setHighlightedGameId(gameId);
  };

  if (loading && patches.length === 0) {
    return <div className="text-center py-12">Loading patches...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap justify-between items-center gap-4">
        <div>
          <h2 className="text-2xl font-bold mb-2">Available Patches</h2>
          <p className="text-muted-foreground">Browse and install community translation patches</p>
        </div>
        <DropdownMenu.Root>
          <DropdownMenu.Trigger asChild>
            <Button variant="outline" disabled={isScanning}>
              <FolderSearch className="mr-2 h-4 w-4" />
              {isScanning ? "Scanning..." : "Scan for Games"}
              <ChevronDown className="ml-2 h-4 w-4" />
            </Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Portal>
            <DropdownMenu.Content
              align="end"
              sideOffset={6}
              className="z-50 min-w-56 rounded-md border bg-popover p-1 text-popover-foreground shadow-md"
            >
              <DropdownMenu.Item
                disabled={isScanning}
                onSelect={() => void handleScanGames(true)}
                className="cursor-pointer rounded-sm px-3 py-2 text-sm outline-none focus:bg-accent focus:text-accent-foreground"
              >
                Automatically find Steam &amp; GOG games
              </DropdownMenu.Item>
              <DropdownMenu.Item
                disabled={isScanning}
                onSelect={() => void handleScanGames(false)}
                className="cursor-pointer rounded-sm px-3 py-2 text-sm outline-none focus:bg-accent focus:text-accent-foreground"
              >
                Choose a folder…
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Portal>
        </DropdownMenu.Root>
      </div>
      <div aria-live="polite">
        {isScanning && <p className="text-sm text-muted-foreground">Scanning game folders…</p>}
        {scanMessage && <p className="text-sm text-muted-foreground">{scanMessage}</p>}
        {scanWarnings.length > 0 && (
          <details className="mt-2 text-sm">
            <summary>Scan warnings ({scanWarnings.length})</summary>
            <ul className="mt-2 list-disc pl-5">
              {scanWarnings.map((warning, index) => (
                <li key={`${index}-${warning}`}>{warning}</li>
              ))}
            </ul>
          </details>
        )}
      </div>

      <AddGameDialog
        open={showAddDialog}
        onOpenChange={(open) => (open ? actions.showAddDialog() : actions.hideAddDialog())}
        onAdded={async () => {
          const updatedGames = (await invoke("get_games_with_patches")) as Game[];
          setGames(updatedGames);
        }}
      />

      <PurchaseGuideDialog
        scummvmId={selectedGuide?.scummvmId || ""}
        gameTitle={selectedGuide?.title || ""}
        description={selectedPatch?.description}
        steamAppId={selectedPatch?.steam_app_id}
        gogId={selectedPatch?.gog_id}
        open={!!selectedGuide}
        onOpenChange={(open) => !open && actions.clearSelectedGuide()}
      />

      {patches.length === 0 ? (
        <Card>
          <CardContent className="pt-6 text-center">
            <p className="text-muted-foreground">
              No patches available. Add patches manually or configure a patch registry.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          {patches.map((patch) => (
            <PatchCard
              key={`${patch.scummvm_id}-${patch.language}-${patch.version}`}
              patch={patch}
              onViewGame={() => patch.game_id && handleViewGame(patch.game_id)}
              onShowGuide={() =>
                actions.setSelectedGuide({ scummvmId: patch.scummvm_id, title: patch.game_title })
              }
            />
          ))}
        </div>
      )}
    </div>
  );
}

type PatchCardProps = {
  patch: PatchWithStatus;
  onViewGame: () => void;
  onShowGuide: () => void;
};

function PatchCard({ patch, onViewGame, onShowGuide }: PatchCardProps) {
  return (
    <Card className="hover:shadow-lg transition-shadow relative">
      <CardContent className="p-0 relative">
        <div className="bg-gradient-to-br from-primary/20 to-secondary/20 rounded-t-lg flex items-center justify-center relative overflow-hidden">
          <GameDetailsBox
            isPatched={patch.patch_installed}
            description={patch.description}
            gameId={patch.scummvm_id}
            coverUrl={patch.cover_url}
            title={patch.game_title}
          />

          <div className="absolute top-2 right-2 flex flex-col gap-1 items-end">
            {patch.patch_installed && (
              <Badge variant="default" className="bg-green-600 backdrop-blur-sm">
                <CheckCircle className="w-3 h-3 mr-1" />
                Installed
              </Badge>
            )}
          </div>
          <DropdownMenu.Root>
            <DropdownMenu.Trigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="absolute top-2 left-2 h-8 w-8 p-0 bg-background/80 backdrop-blur-sm"
              >
                <MoreVertical className="h-4 w-4" />
              </Button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Portal>
              <DropdownMenu.Content
                className="min-w-[160px] rounded-md border bg-background p-1 shadow-lg"
                align="start"
              >
                {patch.game_installed && patch.game_id != null && (
                  <OpenGameFolderMenuItem gameId={patch.game_id} />
                )}
                <DropdownMenu.Item
                  className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                  onSelect={onShowGuide}
                >
                  <ExternalLink className="mr-2 h-4 w-4" />
                  Game Description
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
        </div>
      </CardContent>
      <CardFooter className="flex flex-col gap-2 p-4">
        <div className="w-full">
          <Tooltip.Provider>
            <Tooltip.Root>
              <Tooltip.Trigger asChild>
                <h3 className="font-semibold text-sm truncate mb-1">{patch.game_title}</h3>
              </Tooltip.Trigger>
              <Tooltip.Portal>
                <Tooltip.Content
                  className="rounded-md bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md"
                  sideOffset={5}
                >
                  {patch.game_title}
                </Tooltip.Content>
              </Tooltip.Portal>
            </Tooltip.Root>
          </Tooltip.Provider>
          <p className="text-xs text-muted-foreground truncate">
            v{patch.version} • {patch.author}
          </p>
        </div>

        <StoreInstallButtons
          installed={patch.game_installed}
          scummvmId={patch.scummvm_id}
          steamAppId={patch.steam_app_id}
          gogId={patch.gog_id}
        />

        {patch.patch_installed ? (
          <Button className="w-full" size="sm" onClick={onViewGame}>
            <Play className="mr-2 h-3 w-3" />
            View Game
          </Button>
        ) : patch.game_installed ? (
          <Button className="w-full" size="sm" variant="default" onClick={onViewGame}>
            <Download className="mr-2 h-3 w-3" />
            Install Patch
          </Button>
        ) : (
          <Button className="w-full" size="sm" variant="secondary" onClick={onShowGuide}>
            <ExternalLink className="mr-2 h-3 w-3" />
            Game Description
          </Button>
        )}
      </CardFooter>
    </Card>
  );
}
