import { invoke } from "@tauri-apps/api/core";
import { CheckCircle, Download, Package, Play } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import type { Game } from "@/store/store";

type GameCardProps = {
  game: Game;
};

export default function GameCard({ game }: GameCardProps) {
  const [isInstalling, setIsInstalling] = useState(false);

  const handleLaunch = async () => {
    try {
      await invoke("launch_game", { gameId: game.id });
    } catch (error) {
      console.error("Failed to launch game:", error);
      alert(`Failed to launch game: ${error}`);
    }
  };

  const handleInstallPatch = async () => {
    if (!game.available_patches || game.available_patches.length === 0) return;

    // Install the first available patch (you could expand this to show a selection dialog)
    const patch = game.available_patches[0];

    try {
      setIsInstalling(true);
      await invoke("download_and_install_patch", {
        scummvmId: game.scummvm_id,
        language: patch.language,
        version: patch.version,
      });
      alert("Patch installed successfully!");
      // Reload the page or update state to reflect the new patch status
      window.location.reload();
    } catch (error) {
      console.error("Failed to install patch:", error);
      alert(`Failed to install patch: ${error}`);
    } finally {
      setIsInstalling(false);
    }
  };

  const hasPatch = game.installed_patches && game.installed_patches.length > 0;
  const canInstall = game.has_available_patch && !hasPatch;

  return (
    <Card className="hover:shadow-lg transition-shadow">
      <CardContent className="p-0">
        <div className="aspect-[3/4] bg-gradient-to-br from-primary/20 to-secondary/20 rounded-t-lg flex items-center justify-center">
          {game.cover_url ? (
            <img
              src={game.cover_url}
              alt={game.title}
              className="w-full h-full object-cover rounded-t-lg"
            />
          ) : (
            <Package className="h-16 w-16 text-muted-foreground" />
          )}
        </div>
      </CardContent>
      <CardFooter className="flex flex-col gap-2 p-4">
        <div className="w-full">
          <h3 className="font-semibold text-sm truncate mb-1" title={game.title}>
            {game.title}
          </h3>
          <div className="flex items-center gap-2 flex-wrap">
            <Badge variant="secondary" className="text-xs">
              {game.scummvm_id}
            </Badge>
            {hasPatch && (
              <Badge variant="default" className="text-xs bg-green-600">
                <CheckCircle className="mr-1 h-3 w-3" />
                Patched
              </Badge>
            )}
            {canInstall && (
              <Badge variant="default" className="text-xs bg-yellow-600">
                <Download className="mr-1 h-3 w-3" />
                Patch Available
              </Badge>
            )}
          </div>
        </div>
        <Button className="w-full" size="sm" onClick={handleLaunch}>
          <Play className="mr-2 h-3 w-3" />
          Play
        </Button>
        {canInstall && (
          <Button
            className="w-full"
            size="sm"
            variant="outline"
            onClick={handleInstallPatch}
            disabled={isInstalling}
          >
            <Download className="mr-2 h-3 w-3" />
            {isInstalling ? "Installing..." : "Install Patch"}
          </Button>
        )}
        {hasPatch && (
          <Button className="w-full" size="sm" variant="outline" disabled>
            <CheckCircle className="mr-2 h-3 w-3" />
            Patch Installed
          </Button>
        )}
      </CardFooter>
    </Card>
  );
}
