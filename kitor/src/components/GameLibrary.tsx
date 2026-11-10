import { CollectionView, useGameNavigation, useTranslation } from "@kitor/ui";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ChevronDown, FolderSearch } from "lucide-react";
import { useEffect, useState } from "react";
import GameDetails from "@/components/GameDetails";
import { Button } from "@/components/ui/button";
import { useDialog } from "@/hooks/useDialog";
import { gameSummary } from "@/lib/presentation";
import { useAppStore } from "@/store/store";

type GameLibraryProps = {
  onRefresh: () => void | Promise<void>;
  navigationResetKey?: number;
};

export default function GameLibrary({ onRefresh, navigationResetKey }: GameLibraryProps) {
  const { t } = useTranslation();
  const { games, highlightedGameId, setHighlightedGameId } = useAppStore();
  const navigation = useGameNavigation(navigationResetKey);
  const selected = games.find((game) => String(game.id) === navigation.selectedId);
  const [isScanning, setIsScanning] = useState(false);
  const [isRemoving, setIsRemoving] = useState(false);
  const [checkingUpdates, setCheckingUpdates] = useState(false);
  const [updateMessage, setUpdateMessage] = useState("");
  const checkUpdates = async () => {
    setCheckingUpdates(true);
    try {
      await invoke("refresh_patch_registry");
      await onRefresh();
      setUpdateMessage("Translation catalog refreshed. Available updates appear on your games.");
    } catch (error) {
      setUpdateMessage(
        t("Could not check for translation updates. Your cached catalog is still available. {0}", [
          String(error),
        ])
      );
    } finally {
      setCheckingUpdates(false);
    }
  };
  const { confirm } = useDialog();
  const [scanMessage, setScanMessage] = useState("");
  const [scanWarnings, setScanWarnings] = useState<string[]>([]);

  useEffect(() => {
    if (highlightedGameId !== null) {
      navigation.select(String(highlightedGameId));
      setHighlightedGameId(null);
    }
  }, [highlightedGameId, navigation.select, setHighlightedGameId]);

  const handleScanGames = async (automatic: boolean) => {
    if (isScanning || isRemoving) return;
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
        const selected = await open({
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
      await onRefresh();
    } catch (error) {
      setScanMessage(t("Scan failed: {0}", [String(error)]));
      await onRefresh();
    } finally {
      setIsScanning(false);
    }
  };

  const handleRemoveAllGames = async () => {
    if (isScanning || isRemoving) return;
    setIsRemoving(true);
    try {
      const confirmed = await confirm({
        title: t("Remove all games from your library?"),
        description: t(
          "This only removes tracking in Kitor. Your game files will stay on disk. You can scan your folders to add the games again."
        ),
        confirmText: t("Remove all games"),
        cancelText: t("Cancel"),
      });
      if (!confirmed) return;
      setScanMessage("");
      setScanWarnings([]);
      // Read the full tracked list before removing games.
      const trackedGames = await invoke<{ id: number }[]>("get_all_games");
      for (const game of trackedGames) {
        await invoke("remove_game", { id: game.id });
      }
      setHighlightedGameId(null);
      setScanMessage("All games removed from your library. Game files were left on disk.");
    } catch (error) {
      setScanMessage(t("Could not remove all games: {0}", [String(error)]));
    } finally {
      try {
        await onRefresh();
      } finally {
        setIsRemoving(false);
      }
    }
  };

  const scanButton = (outline = false) => (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <Button
          variant={outline ? "outline" : "default"}
          disabled={isScanning || isRemoving}
          className="text-white"
        >
          <FolderSearch className="mr-2 h-4 w-4" />
          {isRemoving ? t("Removing...") : isScanning ? t("Scanning...") : t("Scan for Games")}
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
          <DropdownMenu.Separator className="my-1 h-px bg-border" />
          <DropdownMenu.Item
            disabled={isScanning || isRemoving}
            onSelect={() => void handleRemoveAllGames()}
            className="cursor-pointer rounded-sm px-3 py-2 text-sm text-destructive outline-none focus:bg-accent data-[disabled]:pointer-events-none data-[disabled]:opacity-50"
          >
            {t("Remove all games")}{" "}
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );

  return (
    <div ref={navigation.root}>
      <div hidden={!!selected}>
        <CollectionView
          collection="library"
          games={games.map(gameSummary)}
          onOpen={navigation.open}
          toolbar={
            <div className="kt-flow-actions">
              <Button variant="outline" onClick={checkUpdates} disabled={checkingUpdates}>
                {checkingUpdates ? t("Checking…") : t("Check Translation Updates")}
              </Button>
              {scanButton(true)}
            </div>
          }
          feedback={
            <div aria-live="polite" className="space-y-2">
              {updateMessage && <p>{updateMessage}</p>}
              {isScanning && <p>{t("Scanning game folders…")}</p>}
              {scanMessage && <p>{t(scanMessage)}</p>}
              {scanWarnings.length > 0 && (
                <details>
                  <summary>
                    {t("Scan warnings (")}
                    {scanWarnings.length})
                  </summary>
                  <ul>
                    {scanWarnings.map((warning, i) => (
                      <li key={`${i}-${warning}`}>{warning}</li>
                    ))}
                  </ul>
                </details>
              )}
            </div>
          }
        />
      </div>
      {selected && (
        <GameDetails
          key={selected.id}
          game={selected}
          collection="library"
          onRemove={onRefresh}
          onBack={navigation.back}
        />
      )}
    </div>
  );
}
