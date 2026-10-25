import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ChevronDown, FolderSearch, Plus } from "lucide-react";
import { useEffect, useState } from "react";
import AddGameDialog from "@/components/AddGameDialog";
import GameCard from "@/components/GameCard";
import { Button } from "@/components/ui/button";
import { useAppStore } from "@/store/store";

type GameLibraryProps = {
  onRefresh: () => void | Promise<void>;
};

export default function GameLibrary({ onRefresh }: GameLibraryProps) {
  const { games, highlightedGameId, setHighlightedGameId } = useAppStore();
  const [isScanning, setIsScanning] = useState(false);
  const [scanMessage, setScanMessage] = useState("");
  const [scanWarnings, setScanWarnings] = useState<string[]>([]);
  const [showAddDialog, setShowAddDialog] = useState(false);

  useEffect(() => {
    if (highlightedGameId) {
      const element = document.getElementById(`game-card-${highlightedGameId}`);
      if (element) {
        element.scrollIntoView({ behavior: "smooth", block: "center" });
        // Clear highlight after animation
        const timer = setTimeout(() => {
          setHighlightedGameId(null);
        }, 2000);
        return () => clearTimeout(timer);
      }
    }
  }, [highlightedGameId, setHighlightedGameId]);

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
        const selected = await open({
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
      await onRefresh();
    } catch (error) {
      setScanMessage(`Scan failed: ${String(error)}`);
      await onRefresh();
    } finally {
      setIsScanning(false);
    }
  };

  const scanButton = (outline = false) => (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <Button
          variant={outline ? "outline" : "default"}
          disabled={isScanning}
          className="text-white"
        >
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
  );

  const handleAddManually = () => setShowAddDialog(true);

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold">Game Library</h2>
          <p className="text-muted-foreground">
            {games.length} {games.length === 1 ? "game" : "games"} in library
          </p>
        </div>
        <div className="flex gap-2">
          {scanButton(true)}
          <Button onClick={handleAddManually}>
            <Plus className="mr-2 h-4 w-4" />
            Add Manually
          </Button>
        </div>
      </div>

      <div aria-live="polite">
        {isScanning && <p className="text-sm text-muted-foreground">Scanning game folders…</p>}
        {scanMessage && <p className="text-sm text-muted-foreground">{scanMessage}</p>}
        {scanWarnings.length > 0 && (
          <details className="mt-2 text-sm">
            <summary>Some locations could not be scanned ({scanWarnings.length})</summary>
            <ul className="mt-2 list-disc pl-5">
              {scanWarnings.map((warning, index) => (
                <li key={`${index}-${warning}`}>{warning}</li>
              ))}
            </ul>
          </details>
        )}
      </div>

      <AddGameDialog open={showAddDialog} onOpenChange={setShowAddDialog} onAdded={onRefresh} />

      {games.length === 0 ? (
        <div className="text-center py-12 border-2 border-dashed rounded-lg">
          <FolderSearch className="mx-auto h-12 w-12 text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">No games with patches found</h3>
          <p className="text-muted-foreground mb-4">
            Scan a folder or add games with available translation patches
          </p>
          {scanButton()}
        </div>
      ) : (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          {games.map((game) => (
            <GameCard
              key={game.id}
              game={game}
              onRemove={onRefresh}
              isHighlighted={game.id === highlightedGameId}
            />
          ))}
        </div>
      )}
    </div>
  );
}
