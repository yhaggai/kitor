import { type Collection, GameDetailsView, useTranslation } from "@kitor/ui";
import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";
import { CatalogGameActions } from "@/components/CatalogGameActions";
import GameCard from "@/components/GameCard";
import { TranslationContributionButton } from "@/components/TranslationContributionButton";
import { TranslationCredit } from "@/components/TranslationCredit";
import { type CatalogGame, catalogGameSummary, patchSummary } from "@/lib/presentation";
import { type Game, type PatchWithStatus, useAppStore } from "@/store/store";

type GameDetailsProps = {
  collection: Collection;
  onBack: () => void;
  onRemove?: () => void;
} & (
  | { game: Game; patch?: never; catalogGame?: never }
  | { game?: never; patch: PatchWithStatus; catalogGame?: never }
  | { game?: never; patch?: never; catalogGame: CatalogGame }
);

// Share an active scan across page changes and React effect remounts.
let installationScan: Promise<void> | undefined;
function scanInstallations() {
  if (!installationScan) {
    installationScan = (async () => {
      await invoke("scan_installed_games");
      const games = await invoke<Game[]>("get_games_with_patches");
      useAppStore.getState().setGames(games);
    })().finally(() => {
      installationScan = undefined;
    });
  }
  return installationScan;
}

/** Both collections resolve actions from the live library, never the entry point. */
export default function GameDetails({
  game,
  patch,
  catalogGame,
  collection,
  onBack,
  onRemove,
}: GameDetailsProps) {
  const { t } = useTranslation();
  const games = useAppStore((state) => state.games);
  const gameId = game?.scummvm_id ?? patch?.scummvm_id ?? catalogGame?.scummvm_id;
  const installed = game ?? games.find((entry) => entry.scummvm_id === gameId);
  const installedId = installed?.id;
  useEffect(() => {
    if (!gameId || installedId !== undefined) return;
    void scanInstallations().catch((error) => {
      console.error("Could not scan Steam and GOG installations", error);
    });
  }, [gameId, installedId]);
  if (installed) {
    return (
      <GameCard
        key={installed.id}
        game={installed}
        collection={collection}
        onBack={onBack}
        onRemove={onRemove}
      />
    );
  }
  if (catalogGame) {
    return (
      <GameDetailsView
        game={catalogGameSummary(catalogGame)}
        collection={collection}
        onBack={onBack}
        actions={
          <div className="space-y-2">
            <CatalogGameActions
              scummvmId={catalogGame.scummvm_id}
              steamAppId={catalogGame.steam_app_id}
            />
            <p className="text-sm text-muted-foreground">
              {t("No compatible patch is available in the current catalog.")}
            </p>
          </div>
        }
      />
    );
  }
  if (!patch) return null;
  return (
    <GameDetailsView
      game={patchSummary({ ...patch, game_installed: false, patch_installed: false })}
      collection={collection}
      onBack={onBack}
      descriptionCredit={<TranslationCredit gameId={patch.scummvm_id} />}
      actions={
        <div className="space-y-2">
          <CatalogGameActions
            scummvmId={patch.scummvm_id}
            steamAppId={patch.steam_app_id}
            gogId={patch.gog_id}
            gogInstallerPath={patch.gog_installer_path}
            addToCartUrl={patch.add_to_cart_url}
          />
          <TranslationContributionButton gameId={patch.scummvm_id} />
        </div>
      }
    />
  );
}
