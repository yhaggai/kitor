import * as AlertDialog from "@radix-ui/react-alert-dialog";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { invoke } from "@tauri-apps/api/core";
import { CheckCircle, Download, MoreVertical, Package, Play, Trash2 } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import type { Game } from "@/store/store";

type GameCardProps = {
  game: Game;
  onRemove?: () => void;
};

export default function GameCard({ game, onRemove }: GameCardProps) {
  const [isInstalling, setIsInstalling] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

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

  const handleRemoveGame = async () => {
    try {
      setIsDeleting(true);
      await invoke("remove_game", { id: game.id });
      setShowDeleteDialog(false);
      onRemove?.();
    } catch (error) {
      console.error("Failed to remove game:", error);
      alert(`Failed to remove game: ${error}`);
    } finally {
      setIsDeleting(false);
    }
  };

  const hasPatch = game.installed_patches && game.installed_patches.length > 0;
  const canInstall = game.has_available_patch && !hasPatch;

  return (
    <>
      <Card className="hover:shadow-lg transition-shadow">
        <CardContent className="p-0 relative">
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
          <DropdownMenu.Root>
            <DropdownMenu.Trigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="absolute top-2 right-2 h-8 w-8 p-0 bg-background/80 backdrop-blur-sm"
              >
                <MoreVertical className="h-4 w-4" />
              </Button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Portal>
              <DropdownMenu.Content
                className="min-w-[160px] rounded-md border bg-background p-1 shadow-lg"
                align="end"
              >
                <DropdownMenu.Item
                  className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm outline-none hover:bg-accent focus:bg-accent"
                  onSelect={handleLaunch}
                >
                  <Play className="mr-2 h-4 w-4" />
                  Play
                </DropdownMenu.Item>
                {canInstall && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm outline-none hover:bg-accent focus:bg-accent"
                    onSelect={handleInstallPatch}
                    disabled={isInstalling}
                  >
                    <Download className="mr-2 h-4 w-4" />
                    {isInstalling ? "Installing..." : "Install Patch"}
                  </DropdownMenu.Item>
                )}
                <DropdownMenu.Separator className="my-1 h-px bg-border" />
                <DropdownMenu.Item
                  className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-destructive outline-none hover:bg-accent focus:bg-accent"
                  onSelect={() => setShowDeleteDialog(true)}
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  Remove Game
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
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

      <AlertDialog.Root open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialog.Portal>
          <AlertDialog.Overlay className="fixed inset-0 bg-black/50" />
          <AlertDialog.Content className="fixed left-1/2 top-1/2 w-[min(420px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-background p-6 shadow-lg">
            <AlertDialog.Title className="text-lg font-semibold">
              Remove {game.title}?
            </AlertDialog.Title>
            <AlertDialog.Description className="mt-2 text-sm text-muted-foreground">
              This will remove the game from your library. The game files on your disk will not be
              deleted.
            </AlertDialog.Description>
            <div className="mt-6 flex justify-end gap-2">
              <AlertDialog.Cancel asChild>
                <Button variant="outline" disabled={isDeleting}>
                  Cancel
                </Button>
              </AlertDialog.Cancel>
              <AlertDialog.Action asChild>
                <Button variant="destructive" onClick={handleRemoveGame} disabled={isDeleting}>
                  <Trash2 className="mr-2 h-4 w-4" />
                  {isDeleting ? "Removing..." : "Remove"}
                </Button>
              </AlertDialog.Action>
            </div>
          </AlertDialog.Content>
        </AlertDialog.Portal>
      </AlertDialog.Root>
    </>
  );
}
