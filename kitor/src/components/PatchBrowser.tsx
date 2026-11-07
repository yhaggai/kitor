import { CollectionView, useGameNavigation, useTranslation } from "@kitor/ui";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { ChevronDown, FolderSearch } from "lucide-react";
import { useEffect, useState } from "react";
import AddGameDialog from "@/components/AddGameDialog";
import GameDetails from "@/components/GameDetails";
import { Button } from "@/components/ui/button";
import { usePatchBrowserState } from "@/hooks/usePatchBrowserState";
import { patchSummary } from "@/lib/presentation";

import { type Game, type Patch, useAppStore } from "@/store/store";

export default function PatchBrowser() {
  const { t } = useTranslation();
  const { state, actions } = usePatchBrowserState();
  const { patches, loading, showAddDialog } = state;

  const { games, setGames } = useAppStore();
  const navigation = useGameNavigation();
  const selected = patches.find((patch) => patchSummary(patch).id === navigation.selectedId);
  const [loadError, setLoadError] = useState<string>();
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
            ? t("No Steam or GOG folders found. Try choosing a folder.")
            : t(
                "Scanned {0} locations. Added {1} games. Only recognized games are added; existing games are skipped.",
                [result.locations.length, result.added_count]
              )
        );
        setScanWarnings(result.warnings);
      } else {
        const selected = await openDialog({
          directory: true,
          multiple: false,
          title: t("Select folder to scan for games"),
        });
        if (!selected) return;
        const added = await invoke<unknown[]>("scan_games", { scanPath: selected });
        setScanMessage(
          t("Added {0} games. Only recognized games are added; existing games are skipped.", [
            added.length,
          ])
        );
      }
    } catch (error) {
      setScanMessage(t("Scan failed: {0}", [String(error)]));
    } finally {
      try {
        setGames(await invoke<Game[]>("get_games_with_patches"));
      } catch (error) {
        setScanWarnings((warnings) => [
          ...warnings,
          t("Could not refresh games: {0}", [String(error)]),
        ]);
      } finally {
        setIsScanning(false);
      }
    }
  };

  useEffect(() => {
    async function loadPatches() {
      try {
        actions.setLoading(true);
        setLoadError(undefined);
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
        setLoadError(String(error));
      } finally {
        actions.setLoading(false);
      }
    }

    loadPatches();
  }, [games, actions]);

  return (
    <div ref={navigation.root}>
      <div hidden={!!selected}>
        <CollectionView
          collection="discovery"
          games={patches.map(patchSummary)}
          loading={loading}
          error={loadError}
          onOpen={navigation.open}
          toolbar={
            <DropdownMenu.Root>
              <DropdownMenu.Trigger asChild>
                <Button variant="outline" disabled={isScanning}>
                  <FolderSearch className="mr-2 h-4 w-4" />
                  {isScanning ? t("Scanning...") : t("Scan for Games")}
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
                    {t("Automatically find Steam & GOG games")}{" "}
                  </DropdownMenu.Item>
                  <DropdownMenu.Item
                    disabled={isScanning}
                    onSelect={() => void handleScanGames(false)}
                    className="cursor-pointer rounded-sm px-3 py-2 text-sm outline-none focus:bg-accent focus:text-accent-foreground"
                  >
                    {t("Choose a folder…")}{" "}
                  </DropdownMenu.Item>
                </DropdownMenu.Content>
              </DropdownMenu.Portal>
            </DropdownMenu.Root>
          }
          feedback={
            <div aria-live="polite">
              {isScanning && (
                <p className="text-sm text-muted-foreground">{t("Scanning game folders…")}</p>
              )}
              {scanMessage && <p className="text-sm text-muted-foreground">{scanMessage}</p>}
              {scanWarnings.length > 0 && (
                <details className="mt-2 text-sm">
                  <summary>
                    {t("Scan warnings (")}
                    {scanWarnings.length})
                  </summary>
                  <ul className="mt-2 list-disc pl-5">
                    {scanWarnings.map((warning, index) => (
                      <li key={`${index}-${warning}`}>{warning}</li>
                    ))}
                  </ul>
                </details>
              )}
            </div>
          }
        />
      </div>

      <AddGameDialog
        open={showAddDialog}
        onOpenChange={(open) => (open ? actions.showAddDialog() : actions.hideAddDialog())}
        onAdded={async () => {
          const updatedGames = (await invoke("get_games_with_patches")) as Game[];
          setGames(updatedGames);
        }}
      />

      {selected && (
        <GameDetails
          patch={selected}
          onRemove={async () => setGames(await invoke<Game[]>("get_games_with_patches"))}
          collection="discovery"
          onBack={navigation.back}
        />
      )}
    </div>
  );
}
