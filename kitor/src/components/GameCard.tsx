import { type Collection, GameDetailsView, useTranslation } from "@kitor/ui";
import * as AlertDialog from "@radix-ui/react-alert-dialog";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import * as Tooltip from "@radix-ui/react-tooltip";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  CheckCircle,
  Download as DownloadIcon,
  Play,
  RefreshCw,
  Settings,
  Trash2,
  Undo,
} from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import BugReportDialog from "@/components/BugReportDialog";
import { DeleteGameDialog } from "@/components/dialogs/DeleteGameDialog";
import { InstallPatchDialog } from "@/components/dialogs/InstallPatchDialog";
import { RestoreGameDialog } from "@/components/dialogs/RestoreGameDialog";
import { ScummVMDialog } from "@/components/dialogs/ScummVMDialog";
import { OpenGameFolderMenuItem } from "@/components/OpenGameFolderMenuItem";
import { TranslationContributionButton } from "@/components/TranslationContributionButton";
import { TranslationCredit } from "@/components/TranslationCredit";
import { Button } from "@/components/ui/button";
import { useDialog } from "@/hooks/useDialog";
import { type PatchDownloadProgress, useGameCardState } from "@/hooks/useGameCardState";
import { primaryGameAction } from "@/lib/gameAction";
import { patchOptionLabel, selectedPatchOption } from "@/lib/patchOptions";
import { gameSummary } from "@/lib/presentation";
import type { Game, GameHealth } from "@/store/store";
import { useAppStore } from "@/store/store";

type GameCardProps = {
  game: Game;
  collection: Collection;
  onRemove?: () => void;
  onBack: () => void;
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

export default function GameCard({ game, collection, onRemove, onBack }: GameCardProps) {
  const { t, language } = useTranslation();
  const displayTitle = language === "he" ? game.title_he?.trim() || game.title : game.title;
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
      // Store uninstallers can leave an empty folder or unrelated files behind.
      // Keep patch recovery records when verification is damaged or uncertain.
      const noInstallation =
        health.game_status === "missing" ||
        (health.game_status === "unrecognized" && health.patch_status === "not_installed");
      if (noInstallation) {
        await invoke("remove_game", { id: game.id });
        const store = useAppStore.getState();
        store.setGames(store.games.filter((entry) => entry.id !== game.id));
        onRemove?.();
        return;
      }
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
      await alert({
        failure: true,
        game: displayTitle,
        title: t("Refresh Failed"),
        description: String(error),
      });
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
        await alert({
          failure: true,
          game: displayTitle,
          title: t("Launch Failed"),
          description: String(error),
        });
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
        updatePatch ? t("Translation updated successfully!") : t("Patch installed successfully!")
      );
    } catch (error) {
      console.error("Failed to install patch:", error);
      const reason = String(error);
      const unrecognizedGameFiles = [
        "Unsupported original version:",
        "Legacy edition checksum mismatch:",
        "Required original is missing:",
      ].some((prefix) => reason.startsWith(prefix));
      const reportDetails = [
        reason,
        `Game ID: ${game.scummvm_id}`,
        `Patch: ${patch.language} ${patch.version}`,
        `Detected version: ${game.detected_version ?? "unknown"}`,
        ...(game.health?.details ?? []),
      ].join("\n");
      await alert({
        failure: true,
        game: displayTitle,
        title: t("Install Failed"),
        description: unrecognizedGameFiles
          ? t("This patch doesn't recognize your installed game files.")
          : reportDetails,
        reportDetails,
      });
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
      await alert({
        failure: true,
        game: displayTitle,
        title: t("Remove Failed"),
        description: String(error),
      });
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
      await alert({
        failure: true,
        game: displayTitle,
        title: t("Restore Failed"),
        description: String(error),
      });
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
        failure: true,
        game: displayTitle,
        title: t("Library Refresh Failed"),
        description: t("{0} The displayed status could not be refreshed: {1}", [
          successMessage,
          String(error),
        ]),
      });
      return;
    }
    await alert({ title: t("Success"), description: successMessage });
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
    : (game.is_patched ?? false);
  const canInstall =
    gameReady &&
    game.has_available_patch &&
    !hasPatch &&
    !recordedPatch &&
    (!game.health || game.health.patch_status === "not_installed");
  const action = primaryGameAction({
    inLibrary: true,
    filesReady: gameReady,
    patched: hasPatch,
    patchAvailable: !!game.has_available_patch,
    purchased: true,
  });
  const showPlay = action === "play" && hasPatch;
  const showInstallPatch = action === "install-patch" && canInstall;
  const showPlayWithout = action === "install-patch" || (action === "play" && !hasPatch);
  const canPlay = showPlay || showPlayWithout;
  const canRestore = gameReady && recordedPatch;
  const canUpdate = gameReady && hasPatch && !!updatePatch;
  const patchBlockReasons = [
    ...(!gameReady ? [`Game files: ${game.health?.game_status}`] : []),
    ...(!game.has_available_patch
      ? ["No compatible patch is available in the current catalog."]
      : []),
    ...(recordedPatch && !hasPatch
      ? ["An existing patch record must be restored or resolved before installation."]
      : []),
    ...(game.health && !["installed", "not_installed"].includes(game.health.patch_status)
      ? [`Patch verification: ${game.health.patch_status}`]
      : []),
  ];
  const blockedPatchDetails = [
    "Patch installation unavailable",
    `Game ID: ${game.scummvm_id}`,
    `Detected version: ${game.detected_version ?? "unknown"}`,
    ...patchBlockReasons,
    ...(game.health?.details ?? []),
    `Available patches: ${game.available_patches?.map((patch) => `${patch.language} ${patch.version}`).join(", ") || "none"}`,
  ].join("\n");
  const busy = refreshing || state.isInstalling || state.isRestoring || state.isDeleting;

  return (
    <>
      <GameDetailsView
        game={gameSummary(game)}
        descriptionCredit={<TranslationCredit gameId={game.scummvm_id} />}
        collection={collection}
        onBack={onBack}
        actions={
          <div className="kt-game-actions">
            <TranslationContributionButton gameId={game.scummvm_id} />
            {refreshing && (
              <p aria-live="polite" className="kt-game-action-status">
                {t("Checking game and patch files…")}
              </p>
            )}
            {game.health &&
              (!gameReady || ["damaged", "unverified"].includes(game.health.patch_status)) && (
                <div aria-live="polite" className="kt-game-action-status">
                  <p className={gameReady ? undefined : "text-destructive"}>
                    {game.health.game_status === "present"
                      ? t("Game files found")
                      : game.health.game_status === "missing"
                        ? t("Game files missing")
                        : t("Game could not be identified")}
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
                    <p key={`${index}-${detail}`}>{detail}</p>
                  ))}
                  <p>
                    {t("Checked")} {new Date(game.health.checked_at).toLocaleTimeString()}
                  </p>
                </div>
              )}
            {!busy && !hasPatch && !canInstall && patchBlockReasons.length > 0 && (
              <BugReportDialog game={displayTitle} details={blockedPatchDetails} />
            )}
            {(game.available_patches?.length ?? 0) > 1 && (
              <div className="w-full space-y-2">
                <label className="block text-sm" htmlFor={`translation-${game.id}`}>
                  {t("Translation")}{" "}
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
                      {t(patchOptionLabel(patch.language, game.scummvm_id))}
                    </option>
                  ))}
                </select>
                {activePatch?.language && (
                  <p className="text-xs text-muted-foreground">
                    {t("Installed:")} {t(patchOptionLabel(activePatch.language, game.scummvm_id))}
                  </p>
                )}

                {changingTranslation && canRestore && (
                  <>
                    <p className="text-xs text-muted-foreground">
                      {t(
                        "Restore the current translation first, then install your selected option."
                      )}{" "}
                    </p>
                    <Button
                      className="w-full"
                      size="sm"
                      variant="outline"
                      disabled={busy}
                      onClick={() => actions.showRestoreDialog()}
                    >
                      <Undo className="mr-2 h-3 w-3" />
                      {t("Restore to change translation")}{" "}
                    </Button>
                  </>
                )}
              </div>
            )}
            {showInstallPatch && (
              <PatchVersionTooltip
                label={t("Available patch version: {0}", [selectedPatch?.version ?? t("Unknown")])}
                disabled={busy}
              >
                <button
                  type="button"
                  className="kt-game-action kt-game-action--solid"
                  onClick={handleInstallPatch}
                  disabled={busy}
                >
                  {state.isInstalling ? t("Installing...") : t("Install Hebrew translation")}
                </button>
              </PatchVersionTooltip>
            )}
            {showPlay && (
              <button
                type="button"
                className="kt-game-action kt-game-action--play"
                onClick={handleLaunch}
                disabled={busy}
              >
                {state.isRestoring ? (
                  <span>
                    {t("Restoring")}{" "}
                    <span className="restore-dots" aria-hidden="true">
                      ...
                    </span>
                  </span>
                ) : (
                  t("Play!")
                )}
              </button>
            )}
            {showPlay && hasPatch && (
              <p className="kt-game-action-note">
                <CheckCircle size={15} aria-hidden="true" />
                {t("With Hebrew translation")}
              </p>
            )}
            {showPlayWithout && (
              <button
                type="button"
                className={`kt-game-action ${showInstallPatch ? "kt-game-action--link" : "kt-game-action--play"}`}
                onClick={handleLaunch}
                disabled={busy}
              >
                {t("Play without translation")}
              </button>
            )}
            {canUpdate && (
              <button
                type="button"
                className="kt-game-action kt-game-action--link"
                onClick={() => setUpdateConfirmationOpen(true)}
                disabled={busy}
              >
                {t("Install Patch Update")}
              </button>
            )}
          </div>
        }
        manageControl={
          <DropdownMenu.Root>
            <DropdownMenu.Trigger asChild>
              <Button
                variant="outline"
                size="sm"
                aria-label={t("Manage game")}
                className="kt-manage-trigger"
              >
                <Settings className="h-5 w-5" />
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
                  {refreshing ? t("Refreshing…") : t("Refresh game")}
                </DropdownMenu.Item>
                {canPlay && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                    onSelect={handleLaunch}
                    disabled={busy}
                  >
                    <Play className="mr-2 h-4 w-4" />
                    {showPlay ? t("Play!") : t("Play without translation")}
                  </DropdownMenu.Item>
                )}
                {showInstallPatch && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                    onSelect={handleInstallPatch}
                    disabled={busy}
                  >
                    <DownloadIcon className="mr-2 h-4 w-4" />
                    {state.isInstalling ? t("Installing...") : t("Install Hebrew translation")}
                  </DropdownMenu.Item>
                )}
                {canRestore && (
                  <DropdownMenu.Item
                    className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                    onSelect={() => actions.showRestoreDialog()}
                    disabled={busy}
                  >
                    <Undo className="mr-2 h-4 w-4" />
                    {t("Restore Game")}{" "}
                  </DropdownMenu.Item>
                )}
                <DropdownMenu.Separator className="my-1 h-px bg-border" />
                <DropdownMenu.Item
                  className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-destructive outline-none hover:bg-accent focus:bg-accent"
                  onSelect={() => actions.showDeleteDialog()}
                  disabled={busy}
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  {t("Remove Game")}{" "}
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
        }
      />

      <AlertDialog.Root open={updateConfirmationOpen} onOpenChange={setUpdateConfirmationOpen}>
        <AlertDialog.Portal>
          <AlertDialog.Overlay className="fixed inset-0 z-50 bg-black/50" />
          <AlertDialog.Content className="fixed left-1/2 top-1/2 z-50 max-h-[85vh] w-[min(520px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-lg border bg-background p-6 shadow-lg">
            <AlertDialog.Title className="text-lg font-semibold">
              {t("Install Patch Update —")} {displayTitle}
            </AlertDialog.Title>
            <AlertDialog.Description className="mt-2 text-sm text-muted-foreground">
              {t("Installed version:")}{" "}
              {game.installed_patches?.find(
                (patch) => patch.status === "active" && patch.language === updatePatch?.language
              )?.version ?? t("Unknown")}
              {" → "}
              {t("New version:")} {updatePatch?.version ?? t("Unavailable")}
              {t(". Close the game before confirming the update.")}{" "}
            </AlertDialog.Description>
            <div className="mt-6 flex justify-end gap-2">
              <AlertDialog.Cancel asChild>
                <Button variant="outline">{t("Cancel")}</Button>
              </AlertDialog.Cancel>
              <AlertDialog.Action asChild>
                <Button onClick={handleInstallPatch} disabled={busy || !canUpdate}>
                  {t("Confirm")}{" "}
                </Button>
              </AlertDialog.Action>
            </div>
          </AlertDialog.Content>
        </AlertDialog.Portal>
      </AlertDialog.Root>

      <DeleteGameDialog
        gameTitle={displayTitle}
        open={state.showDeleteDialog}
        onOpenChange={(open) => (open ? actions.showDeleteDialog() : actions.hideDeleteDialog())}
        onConfirm={handleRemoveGame}
        isDeleting={state.isDeleting}
      />

      <RestoreGameDialog
        gameTitle={displayTitle}
        open={state.showRestoreDialog}
        onOpenChange={(open) => (open ? actions.showRestoreDialog() : actions.hideRestoreDialog())}
        onConfirm={handleRestorePatch}
        isRestoring={state.isRestoring}
      />

      <InstallPatchDialog
        updating={!!updatePatch}
        gameTitle={displayTitle}
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
