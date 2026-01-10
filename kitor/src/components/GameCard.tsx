import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import * as Tooltip from "@radix-ui/react-tooltip";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  CheckCircle,
  Download as DownloadIcon,
  MoreVertical,
  Package,
  Play,
  Trash2,
  Undo,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { DeleteGameDialog } from "@/components/dialogs/DeleteGameDialog";
import { InstallPatchDialog } from "@/components/dialogs/InstallPatchDialog";
import { RestoreGameDialog } from "@/components/dialogs/RestoreGameDialog";
import { ScummVMDialog } from "@/components/dialogs/ScummVMDialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import { useDialog } from "@/hooks/useDialog";
import { useGameCardState } from "@/hooks/useGameCardState";
import { getCoverSrc } from "@/lib/utils";
import type { Game } from "@/store/store";
import { useAppStore } from "@/store/store";

type GameCardProps = {
  game: Game;
  onRemove?: () => void;
  isHighlighted?: boolean;
};

export default function GameCard({ game, onRemove, isHighlighted }: GameCardProps) {
  const { state, actions } = useGameCardState();
  const [coverError, setCoverError] = useState(false);
  const { isScummVMReady } = useAppStore();
  const { alert } = useDialog();

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
    if (coverError) return null;
    return getCoverSrc(game.cover_url);
  }, [coverError, game.cover_url]);

  const handleLaunch = async () => {
    if (!isScummVMReady) {
      actions.showScummVMDialog();
      return;
    }

    try {
      await invoke("launch_game", { gameId: game.id });
    } catch (error) {
      console.error("Failed to launch game:", error);

      // Check if error is about ScummVM not being downloaded
      const errorStr = String(error);
      if (errorStr.includes("not downloaded") || errorStr.includes("not found")) {
        actions.showScummVMDialog();
      } else {
        await alert({ title: "Launch Failed", description: String(error) });
      }
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
      await alert({ title: "Success", description: "Patch installed successfully!" });
      // Reload the page or update state to reflect the new patch status
      window.location.reload();
    } catch (error) {
      console.error("Failed to install patch:", error);
      await alert({ title: "Install Failed", description: String(error) });
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
      await alert({ title: "Remove Failed", description: String(error) });
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
      await alert({ title: "Success", description: "Game restored successfully!" });
      window.location.reload();
    } catch (error) {
      console.error("Failed to restore game:", error);
      await alert({ title: "Restore Failed", description: String(error) });
    } finally {
      actions.finishRestore();
    }
  };

  const hasPatch = game.installed_patches && game.installed_patches.length > 0;
  const canInstall = game.has_available_patch && !hasPatch;
  const canRestore = hasPatch;

  return (
    <>
      <Card
        id={`game-card-${game.id}`}
        className={`hover:shadow-lg transition-all duration-300 ${
          isHighlighted ? "ring-2 ring-primary shadow-xl scale-[1.02]" : ""
        }`}
      >
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
                    <DownloadIcon className="mr-2 h-4 w-4" />
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
                  <DownloadIcon className="mr-1 h-3 w-3" />
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
              <DownloadIcon className="mr-2 h-3 w-3" />
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

      <DeleteGameDialog
        gameTitle={game.title}
        open={state.showDeleteDialog}
        onOpenChange={(open) => (open ? actions.showDeleteDialog() : actions.hideDeleteDialog())}
        onConfirm={handleRemoveGame}
        isDeleting={state.isDeleting}
      />

      <RestoreGameDialog
        gameTitle={game.title}
        open={state.showRestoreDialog}
        onOpenChange={(open) => (open ? actions.showRestoreDialog() : actions.hideRestoreDialog())}
        onConfirm={handleRestorePatch}
        isRestoring={state.isRestoring}
      />

      <InstallPatchDialog
        gameTitle={game.title}
        open={state.showInstallDialog}
        onOpenChange={(open) => (open ? actions.showInstallDialog() : actions.hideInstallDialog())}
        installSteps={state.installSteps}
      />

      <ScummVMDialog
        open={state.showScummVMDialog}
        onOpenChange={(open) => (open ? actions.showScummVMDialog() : actions.hideScummVMDialog())}
      />
    </>
  );
}
