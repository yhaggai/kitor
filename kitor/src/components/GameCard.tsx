import * as AlertDialog from "@radix-ui/react-alert-dialog";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import * as Tooltip from "@radix-ui/react-tooltip";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  CheckCircle,
  Download,
  Loader2,
  MoreVertical,
  Package,
  Play,
  Trash2,
  Undo,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import { useGameCardState } from "@/hooks/useGameCardState";
import type { Game } from "@/store/store";

type GameCardProps = {
  game: Game;
  onRemove?: () => void;
};

export default function GameCard({ game, onRemove }: GameCardProps) {
  const { state, actions } = useGameCardState();
  const [coverError, setCoverError] = useState(false);

  useEffect(() => {
    console.log("[GameCard] Setting up install-progress listener for", game.scummvm_id);

    const unlisten = listen<{ scummvm_id: string; progress: number; step: string }>(
      "install-progress",
      (event) => {
        console.log("[GameCard] Received event:", event.payload);
        if (event.payload.scummvm_id === game.scummvm_id) {
          console.log("[GameCard] Updating progress:", event.payload.progress, event.payload.step);
          actions.updateProgress(event.payload.progress, event.payload.step);
        }
      }
    );

    return () => {
      console.log("[GameCard] Cleaning up listener for", game.scummvm_id);
      unlisten.then((fn) => fn());
    };
  }, [game.scummvm_id, actions]);

  const coverSrc = useMemo(() => {
    if (!game.cover_url || coverError) return null;
    if (game.cover_url.startsWith("http://") || game.cover_url.startsWith("https://")) {
      console.log("[cover] Using remote URL:", game.cover_url);
      return game.cover_url;
    }
    const normalizedPath = game.cover_url.split("\\").join("/");
    const assetUrl = convertFileSrc(normalizedPath);
    console.log("[cover] Local path:", game.cover_url, "→ asset URL:", assetUrl);
    return assetUrl;
  }, [coverError, game.cover_url]);

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
      actions.showInstallDialog();
      actions.startInstall();
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
      actions.finishInstall();
    }
  };

  const handleRemoveGame = async () => {
    try {
      actions.startDelete();
      await invoke("remove_game", { id: game.id });
      actions.hideDeleteDialog();
      onRemove?.();
    } catch (error) {
      console.error("Failed to remove game:", error);
      alert(`Failed to remove game: ${error}`);
    } finally {
      actions.finishDelete();
    }
  };

  const handleRestorePatch = async () => {
    if (!game.installed_patches || game.installed_patches.length === 0) return;

    const installedPatch = game.installed_patches[0];

    try {
      actions.startRestore();
      await invoke("restore_backup", {
        installedPatchId: installedPatch.id,
      });
      alert("Game restored successfully!");
      window.location.reload();
    } catch (error) {
      console.error("Failed to restore game:", error);
      alert(`Failed to restore game: ${error}`);
    } finally {
      actions.finishRestore();
    }
  };

  const hasPatch = game.installed_patches && game.installed_patches.length > 0;
  const canInstall = game.has_available_patch && !hasPatch;
  const canRestore = hasPatch;

  return (
    <>
      <Card className="hover:shadow-lg transition-shadow">
        <CardContent className="p-0 relative">
          <div className="aspect-[3/4] bg-gradient-to-br from-primary/20 to-secondary/20 rounded-t-lg flex items-center justify-center">
            {coverSrc ? (
              <img
                src={coverSrc}
                alt={game.title}
                className="w-full h-full object-cover rounded-t-lg"
                loading="lazy"
                onError={() => setCoverError(true)}
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
                {hasPatch && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                    onSelect={handleLaunch}
                  >
                    <Play className="mr-2 h-4 w-4" />
                    Play
                  </DropdownMenu.Item>
                )}
                {canInstall && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                    onSelect={handleInstallPatch}
                    disabled={state.isInstalling}
                  >
                    <Download className="mr-2 h-4 w-4" />
                    {state.isInstalling ? "Installing..." : "Install Patch"}
                  </DropdownMenu.Item>
                )}
                {canRestore && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                    onSelect={() => actions.showRestoreDialog()}
                  >
                    <Undo className="mr-2 h-4 w-4" />
                    Restore Game
                  </DropdownMenu.Item>
                )}
                <DropdownMenu.Separator className="my-1 h-px bg-border" />
                <DropdownMenu.Item
                  className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-destructive outline-none hover:bg-accent focus:bg-accent"
                  onSelect={() => actions.showDeleteDialog()}
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
            <Tooltip.Provider>
              <Tooltip.Root>
                <Tooltip.Trigger asChild>
                  <h3 className="font-semibold text-sm truncate mb-1">{game.title}</h3>
                </Tooltip.Trigger>
                <Tooltip.Portal>
                  <Tooltip.Content
                    className="rounded-md bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md"
                    sideOffset={5}
                  >
                    {game.title}
                  </Tooltip.Content>
                </Tooltip.Portal>
              </Tooltip.Root>
            </Tooltip.Provider>
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
          {hasPatch && (
            <Button className="w-full" size="sm" onClick={handleLaunch}>
              <Play className="mr-2 h-3 w-3" />
              Play
            </Button>
          )}
          {canInstall && (
            <Button className="w-full" size="sm" variant="outline" onClick={handleInstallPatch}>
              <Download className="mr-2 h-3 w-3" />
              Install Patch
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

      <AlertDialog.Root
        open={state.showDeleteDialog}
        onOpenChange={(open) => (open ? actions.showDeleteDialog() : actions.hideDeleteDialog())}
      >
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
                <Button variant="outline" disabled={state.isDeleting} className="text-foreground">
                  Cancel
                </Button>
              </AlertDialog.Cancel>
              <AlertDialog.Action asChild>
                <Button
                  variant="destructive"
                  onClick={handleRemoveGame}
                  disabled={state.isDeleting}
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  {state.isDeleting ? "Removing..." : "Remove"}
                </Button>
              </AlertDialog.Action>
            </div>
          </AlertDialog.Content>
        </AlertDialog.Portal>
      </AlertDialog.Root>

      <AlertDialog.Root
        open={state.showRestoreDialog}
        onOpenChange={(open) => (open ? actions.showRestoreDialog() : actions.hideRestoreDialog())}
      >
        <AlertDialog.Portal>
          <AlertDialog.Overlay className="fixed inset-0 bg-black/50" />
          <AlertDialog.Content className="fixed left-1/2 top-1/2 w-[min(420px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-background p-6 shadow-lg">
            <AlertDialog.Title className="text-lg font-semibold">
              Restore {game.title}?
            </AlertDialog.Title>
            <AlertDialog.Description className="mt-2 text-sm text-muted-foreground">
              This will restore the game to its original state before the patch was installed. The
              patch will be removed and the backup files will be restored.
            </AlertDialog.Description>
            <div className="mt-6 flex justify-end gap-2">
              <AlertDialog.Cancel asChild>
                <Button variant="outline" disabled={state.isRestoring} className="text-foreground">
                  Cancel
                </Button>
              </AlertDialog.Cancel>
              <AlertDialog.Action asChild>
                <Button onClick={handleRestorePatch} disabled={state.isRestoring}>
                  <Undo className="mr-2 h-4 w-4" />
                  {state.isRestoring ? "Restoring..." : "Restore"}
                </Button>
              </AlertDialog.Action>
            </div>
          </AlertDialog.Content>
        </AlertDialog.Portal>
      </AlertDialog.Root>

      <AlertDialog.Root
        open={state.showInstallDialog}
        onOpenChange={(open) => (open ? actions.showInstallDialog() : actions.hideInstallDialog())}
      >
        <AlertDialog.Portal>
          <AlertDialog.Overlay className="fixed inset-0 bg-black/50" />
          <AlertDialog.Content className="fixed left-1/2 top-1/2 w-[min(500px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-background p-6 shadow-lg">
            <AlertDialog.Title className="text-lg font-semibold mb-4">
              Installing Patch for {game.title}
            </AlertDialog.Title>
            <div className="space-y-3">
              {state.installSteps.map((step) => (
                <div key={step.id} className="flex items-center gap-3 py-2">
                  <div className="flex-shrink-0 w-5 h-5">
                    {step.status === "complete" && (
                      <CheckCircle className="w-5 h-5 text-green-600" />
                    )}
                    {step.status === "in-progress" && (
                      <Loader2 className="w-5 h-5 text-primary animate-spin" />
                    )}
                    {step.status === "pending" && <div className="w-5 h-5" />}
                  </div>
                  <span
                    className={
                      step.status === "pending"
                        ? "text-muted-foreground"
                        : step.status === "complete"
                          ? "text-foreground"
                          : "text-foreground font-medium"
                    }
                  >
                    {step.label}
                  </span>
                </div>
              ))}
            </div>
            <div className="mt-6 flex justify-end">
              <AlertDialog.Cancel asChild>
                <Button variant="outline" className="text-foreground">
                  Close
                </Button>
              </AlertDialog.Cancel>
            </div>
          </AlertDialog.Content>
        </AlertDialog.Portal>
      </AlertDialog.Root>
    </>
  );
}
