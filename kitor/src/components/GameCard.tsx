import * as AlertDialog from "@radix-ui/react-alert-dialog";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import * as Tooltip from "@radix-ui/react-tooltip";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  CheckCircle,
  Download as DownloadIcon,
  MoreVertical,
  Play,
  RefreshCw,
  Trash2,
  Undo,
} from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import { DeleteGameDialog } from "@/components/dialogs/DeleteGameDialog";
import { InstallPatchDialog } from "@/components/dialogs/InstallPatchDialog";
import { RestoreGameDialog } from "@/components/dialogs/RestoreGameDialog";
import { ScummVMDialog } from "@/components/dialogs/ScummVMDialog";
import { GameDetailsBox } from "@/components/GameDetailsBox";
import { OpenGameFolderMenuItem } from "@/components/OpenGameFolderMenuItem";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import { useDialog } from "@/hooks/useDialog";
import { type PatchDownloadProgress, useGameCardState } from "@/hooks/useGameCardState";
import { patchOptionLabel, selectedPatchOption } from "@/lib/patchOptions";
import type { Game, GameHealth } from "@/store/store";
import { useAppStore } from "@/store/store";

type GameCardProps = {
  game: Game;
  onRemove?: () => void;
  isHighlighted?: boolean;
};

function PatchVersionTooltip({
  children,
  label,
  disabled,
}: {
  children: ReactNode;
  label: string;
  disabled: boolean;
}) {
  return (
    <Tooltip.Provider>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          <span className="block w-full" tabIndex={disabled ? 0 : undefined}>
            {children}
          </span>
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            className="z-50 max-w-xs rounded-md bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md"
            sideOffset={5}
          >
            {label}
            <Tooltip.Arrow className="fill-popover" />
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}

export default function GameCard({ game, onRemove, isHighlighted }: GameCardProps) {
  const [refreshing, setRefreshing] = useState(false);
  const [updateConfirmationOpen, setUpdateConfirmationOpen] = useState(false);
  const [translationChoice, setTranslationChoice] = useState("");
  const { state, actions } = useGameCardState();
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

    const unlistenDownload = listen<PatchDownloadProgress>("patch-download-progress", (event) => {
      if (event.payload.scummvm_id === game.scummvm_id) {
        actions.updateDownload(event.payload);
      }
    });

    return () => {
      unlistenDownload.then((fn) => fn());
      console.log("[GameCard] Cleaning up listener for", game.scummvm_id);
      unlisten.then((fn) => fn());
    };
  }, [game.scummvm_id, actions]);

  const handleRefresh = async () => {
    if (refreshing) return;
    setRefreshing(true);
    try {
      const health = await invoke<GameHealth>("refresh_game", { gameId: game.id });
      const games = await invoke<Game[]>("get_games_with_patches");
      const updatedGame = games.find((entry) => entry.id === game.id);
      if (!updatedGame) throw new Error("Updated game was not found in the library.");

      // Disk health alone cannot clear stale installation records in the UI.
      // Reload patch availability and installed patches along with the check.
      const store = useAppStore.getState();
      store.setGames(
        store.games.map((entry) => (entry.id === game.id ? { ...updatedGame, health } : entry))
      );
    } catch (error) {
      await alert({ title: "Refresh Failed", description: String(error) });
    } finally {
      setRefreshing(false);
    }
  };

  const handleLaunch = async () => {
    if (game.requires_scummvm !== false && !isScummVMReady) {
      actions.showScummVMDialog();
      return;
    }

    try {
      await invoke("launch_game", { gameId: game.id });
    } catch (error) {
      console.error("Failed to launch game:", error);

      // Check if error is about ScummVM not being downloaded
      const errorStr = String(error);
      if (
        game.requires_scummvm !== false &&
        (errorStr.includes("not downloaded") || errorStr.includes("not found"))
      ) {
        actions.showScummVMDialog();
      } else {
        await alert({ title: "Launch Failed", description: String(error) });
      }
    }
  };

  const handleInstallPatch = async () => {
    if (!game.available_patches || game.available_patches.length === 0) return;

    const patch = selectedPatch;
    if (!patch) return;

    try {
      actions.showInstallDialog();
      actions.startInstall();
      await invoke("download_and_install_patch", {
        scummvmId: game.scummvm_id,
        language: patch.language,
        version: patch.version,
      });
      actions.hideInstallDialog();
      await refreshPatchStatus(
        updatePatch ? "Translation updated successfully!" : "Patch installed successfully!"
      );
    } catch (error) {
      console.error("Failed to install patch:", error);
      await alert({ title: "Install Failed", description: String(error) });
    } finally {
      actions.hideInstallDialog();
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

    const installedPatch =
      game.installed_patches.find((patch) => patch.status === "active") ??
      game.installed_patches[0];

    try {
      actions.startRestore();
      await invoke("restore_backup", {
        installedPatchId: installedPatch.id,
      });
      actions.hideRestoreDialog();
      await refreshPatchStatus("Game restored successfully!");
    } catch (error) {
      console.error("Failed to restore game:", error);
      await alert({ title: "Restore Failed", description: String(error) });
    } finally {
      actions.finishRestore();
    }
  };

  const refreshPatchStatus = async (successMessage: string) => {
    try {
      const games = await invoke<Game[]>("get_games_with_patches");
      const updatedGame = games.find((entry) => entry.id === game.id);
      if (!updatedGame) throw new Error("Updated game was not found in the library.");

      // Keep the list mounted and preserve its order and the other cards' state.
      // Replace stale health too: the previous check predates this operation.
      const store = useAppStore.getState();
      store.setGames(store.games.map((entry) => (entry.id === game.id ? updatedGame : entry)));
    } catch (error) {
      console.error("Failed to refresh patch status:", error);
      await alert({
        title: "Library Refresh Failed",
        description: `${successMessage} The displayed status could not be refreshed: ${String(error)}`,
      });
      return;
    }
    await alert({ title: "Success", description: successMessage });
  };

  const recordedPatch = !!game.installed_patches?.length;
  const selectedPatch = selectedPatchOption(
    game.available_patches ?? [],
    game.installed_patches ?? [],
    translationChoice
  );
  const activePatch = game.installed_patches?.find((patch) => patch.status === "active");
  const changingTranslation =
    !!activePatch && !!selectedPatch && activePatch.language !== selectedPatch.language;
  const updatePatch = game.available_patches?.find(
    (available) =>
      available.language === selectedPatch?.language &&
      game.installed_patches?.some(
        (installed) =>
          installed.status === "active" &&
          installed.language === available.language &&
          available.release_revision > installed.release_revision &&
          available.version !== installed.version
      )
  );
  const gameReady = !game.health || game.health.game_status === "present";
  const hasPatch = game.health
    ? game.health.patch_status === "installed"
    : (game.is_patched ?? recordedPatch);
  const canPlay = gameReady;
  const canInstall =
    gameReady &&
    game.has_available_patch &&
    !hasPatch &&
    !recordedPatch &&
    (!game.health || game.health.patch_status === "not_installed");
  const canRestore = gameReady && recordedPatch;
  const canUpdate = gameReady && hasPatch && !!updatePatch;
  const busy = refreshing || state.isInstalling || state.isRestoring || state.isDeleting;

  return (
    <>
      <Card
        id={`game-card-${game.id}`}
        className={`hover:shadow-lg transition-all duration-300 ${
          isHighlighted ? "ring-2 ring-primary shadow-xl scale-[1.02]" : ""
        }`}
      >
        <CardContent className="p-0 relative">
          <div className="bg-gradient-to-br from-primary/20 to-secondary/20 rounded-t-lg flex items-center justify-center">
            <GameDetailsBox
              isPatched={hasPatch}
              gameId={game.scummvm_id}
              coverUrl={game.cover_url}
              title={game.title}
              description={
                game.available_patches?.find((patch) => patch.description?.trim())?.description
              }
            />
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
                <OpenGameFolderMenuItem gameId={game.id} />
                <DropdownMenu.Item
                  className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent data-[disabled]:opacity-50"
                  onSelect={handleRefresh}
                  disabled={busy}
                >
                  <RefreshCw className={`mr-2 h-4 w-4 ${refreshing ? "animate-spin" : ""}`} />
                  {refreshing ? "Refreshing…" : "Refresh game"}
                </DropdownMenu.Item>
                {canPlay && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                    onSelect={handleLaunch}
                    disabled={busy}
                  >
                    <Play className="mr-2 h-4 w-4" />
                    Play
                  </DropdownMenu.Item>
                )}
                {canInstall && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                    onSelect={handleInstallPatch}
                    disabled={busy}
                  >
                    <DownloadIcon className="mr-2 h-4 w-4" />
                    {state.isInstalling ? "Installing..." : "Install Patch"}
                  </DropdownMenu.Item>
                )}
                {canRestore && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                    onSelect={() => actions.showRestoreDialog()}
                    disabled={busy}
                  >
                    <Undo className="mr-2 h-4 w-4" />
                    Restore Game
                  </DropdownMenu.Item>
                )}
                <DropdownMenu.Separator className="my-1 h-px bg-border" />
                <DropdownMenu.Item
                  className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-destructive outline-none hover:bg-accent focus:bg-accent"
                  onSelect={() => actions.showDeleteDialog()}
                  disabled={busy}
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
                  {canUpdate ? (
                    <DownloadIcon className="mr-1 h-3 w-3" />
                  ) : (
                    <CheckCircle className="mr-1 h-3 w-3" />
                  )}
                  {canUpdate ? "Patch Update Available" : "Patched"}
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
          {refreshing && (
            <p aria-live="polite" className="text-xs text-muted-foreground">
              Checking game and patch files…
            </p>
          )}
          {game.health && (
            <div aria-live="polite" className="w-full space-y-1 text-xs">
              <p className={gameReady ? "text-muted-foreground" : "text-destructive"}>
                {game.health.game_status === "present"
                  ? "Game files found"
                  : game.health.game_status === "missing"
                    ? "Game files missing"
                    : "Game could not be identified"}
                {" · "}
                {
                  {
                    installed: "Patch verified",
                    not_installed: "No recorded patch",
                    damaged: "Patch files missing or changed",
                    unverified: "Patch could not be verified",
                  }[game.health.patch_status]
                }
              </p>
              {game.health.details.map((detail, index) => (
                <p key={`${index}-${detail}`} className="break-words text-muted-foreground">
                  {detail}
                </p>
              ))}
              <p className="text-muted-foreground">
                Checked {new Date(game.health.checked_at).toLocaleTimeString()}
              </p>
            </div>
          )}
          {canPlay && (
            <Button className="w-full" size="sm" onClick={handleLaunch} disabled={busy}>
              <Play className="mr-2 h-3 w-3" />
              Play
            </Button>
          )}
          {(game.available_patches?.length ?? 0) > 1 && (
            <div className="w-full space-y-2">
              <label className="block text-sm" htmlFor={`translation-${game.id}`}>
                Translation
              </label>
              <select
                id={`translation-${game.id}`}
                className="w-full rounded-md border bg-background px-3 py-2 text-sm"
                value={selectedPatch?.language ?? ""}
                onChange={(event) => setTranslationChoice(event.target.value)}
                disabled={busy}
              >
                {game.available_patches?.map((patch) => (
                  <option key={patch.language} value={patch.language}>
                    {patchOptionLabel(patch.language, game.scummvm_id)}
                  </option>
                ))}
              </select>
              {activePatch?.language && (
                <p className="text-xs text-muted-foreground">
                  Installed: {patchOptionLabel(activePatch.language, game.scummvm_id)}
                </p>
              )}
              <p className="text-xs text-muted-foreground">{selectedPatch?.description}</p>
              {changingTranslation && canRestore && (
                <>
                  <p className="text-xs text-muted-foreground">
                    Restore the current translation first, then install your selected option.
                  </p>
                  <Button
                    className="w-full"
                    size="sm"
                    variant="outline"
                    disabled={busy}
                    onClick={() => actions.showRestoreDialog()}
                  >
                    <Undo className="mr-2 h-3 w-3" />
                    Restore to change translation
                  </Button>
                </>
              )}
            </div>
          )}
          {canInstall && (
            <PatchVersionTooltip
              label={`Available patch version: ${selectedPatch?.version ?? "Unknown"}`}
              disabled={busy}
            >
              <Button
                className="w-full"
                size="sm"
                variant="outline"
                onClick={handleInstallPatch}
                disabled={busy}
              >
                <DownloadIcon className="mr-2 h-3 w-3" />
                Install Patch
              </Button>
            </PatchVersionTooltip>
          )}
          {canUpdate && (
            <div className="w-full space-y-2">
              <p className="text-xs text-muted-foreground">
                Translation update:{" "}
                {game.installed_patches?.find((p) => p.language === updatePatch?.language)?.version}{" "}
                → {updatePatch?.version}
              </p>
              <PatchVersionTooltip
                label={`Installed version: ${game.installed_patches?.find((patch) => patch.status === "active" && patch.language === updatePatch?.language)?.version ?? "Unknown"} → Available version: ${updatePatch?.version ?? "Unknown"}`}
                disabled={busy}
              >
                <Button
                  className="w-full"
                  size="sm"
                  onClick={() => setUpdateConfirmationOpen(true)}
                  disabled={busy}
                >
                  <DownloadIcon className="mr-2 h-3 w-3" />
                  Install Patch Update
                </Button>
              </PatchVersionTooltip>
            </div>
          )}
          {hasPatch && !canUpdate && !changingTranslation && (
            <PatchVersionTooltip
              label={`Installed patch version: ${game.installed_patches?.find((patch) => patch.status === "active")?.version ?? "Unknown"}`}
              disabled
            >
              <Button className="w-full" size="sm" variant="outline" disabled>
                <CheckCircle className="mr-2 h-3 w-3" />
                Patch Installed
              </Button>
            </PatchVersionTooltip>
          )}
        </CardFooter>
      </Card>

      <AlertDialog.Root open={updateConfirmationOpen} onOpenChange={setUpdateConfirmationOpen}>
        <AlertDialog.Portal>
          <AlertDialog.Overlay className="fixed inset-0 z-50 bg-black/50" />
          <AlertDialog.Content className="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[min(520px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border bg-background p-6 shadow-lg">
            <AlertDialog.Title className="text-lg font-semibold">
              Install Patch Update — {game.title}
            </AlertDialog.Title>
            <AlertDialog.Description className="mt-2 text-sm text-muted-foreground">
              Installed version:{" "}
              {game.installed_patches?.find(
                (patch) => patch.status === "active" && patch.language === updatePatch?.language
              )?.version ?? "Unknown"}
              {" → "}New version: {updatePatch?.version ?? "Unavailable"}. Close the game before
              confirming the update.
            </AlertDialog.Description>
            <h4 className="mt-4 font-medium">What changed</h4>
            <p className="mt-2 whitespace-pre-wrap break-words text-sm text-muted-foreground">
              {updatePatch?.description?.trim() ||
                "No release notes were provided for this update."}
            </p>
            <div className="mt-6 flex justify-end gap-2">
              <AlertDialog.Cancel asChild>
                <Button variant="outline">Cancel</Button>
              </AlertDialog.Cancel>
              <AlertDialog.Action asChild>
                <Button onClick={handleInstallPatch} disabled={busy || !canUpdate}>
                  Confirm
                </Button>
              </AlertDialog.Action>
            </div>
          </AlertDialog.Content>
        </AlertDialog.Portal>
      </AlertDialog.Root>

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
        updating={!!updatePatch}
        gameTitle={game.title}
        open={state.showInstallDialog}
        onOpenChange={(open) => (open ? actions.showInstallDialog() : actions.hideInstallDialog())}
        installSteps={state.installSteps}
        progress={state.installProgress}
        download={state.patchDownload}
      />

      <ScummVMDialog
        open={state.showScummVMDialog}
        onOpenChange={(open) => (open ? actions.showScummVMDialog() : actions.hideScummVMDialog())}
      />
    </>
  );
}
