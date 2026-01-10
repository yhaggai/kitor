import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderSearch, Plus } from "lucide-react";
import { useEffect, useState } from "react";
import AddGameDialog from "@/components/AddGameDialog";
import GameCard from "@/components/GameCard";
import { Button } from "@/components/ui/button";
import { useAppStore } from "@/store/store";

type GameLibraryProps = {
  onRefresh: () => void;
};

export default function GameLibrary({ onRefresh }: GameLibraryProps) {
  const { games, highlightedGameId, setHighlightedGameId } = useAppStore();
  const [isScanning, setIsScanning] = useState(false);
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

  const handleScanGames = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select folder to scan for games",
      });

      if (selected) {
        setIsScanning(true);
        await invoke("scan_games", { scanPath: selected });
        await onRefresh();
      }
    } catch (error) {
      console.error("Failed to scan games:", error);
    } finally {
      setIsScanning(false);
    }
  };

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
          <Button variant="outline" onClick={handleScanGames} disabled={isScanning}>
            <FolderSearch className="mr-2 h-4 w-4" />
            {isScanning ? "Scanning..." : "Scan Folder"}
          </Button>
          <Button onClick={handleAddManually}>
            <Plus className="mr-2 h-4 w-4" />
            Add Manually
          </Button>
        </div>
      </div>

      <AddGameDialog open={showAddDialog} onOpenChange={setShowAddDialog} onAdded={onRefresh} />

      {games.length === 0 ? (
        <div className="text-center py-12 border-2 border-dashed rounded-lg">
          <FolderSearch className="mx-auto h-12 w-12 text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">No games with patches found</h3>
          <p className="text-muted-foreground mb-4">
            Scan a folder or add games with available translation patches
          </p>
          <Button onClick={handleScanGames}>
            <FolderSearch className="mr-2 h-4 w-4" />
            Scan for Games
          </Button>
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
