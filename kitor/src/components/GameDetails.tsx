import { type Collection, GameDetailsView } from "@kitor/ui";
import GameCard from "@/components/GameCard";
import { StoreInstallButtons } from "@/components/StoreInstallButtons";
import { TranslationContributionButton } from "@/components/TranslationContributionButton";
import { TranslationCredit } from "@/components/TranslationCredit";
import { patchSummary } from "@/lib/presentation";
import { type Game, type PatchWithStatus, useAppStore } from "@/store/store";

type GameDetailsProps = {
  collection: Collection;
  onBack: () => void;
  onRemove?: () => void;
} & ({ game: Game; patch?: never } | { game?: never; patch: PatchWithStatus });

/** Both collections resolve actions from the live library, never the entry point. */
export default function GameDetails({
  game,
  patch,
  collection,
  onBack,
  onRemove,
}: GameDetailsProps) {
  const games = useAppStore((state) => state.games);
  const installed = game ?? games.find((entry) => entry.scummvm_id === patch?.scummvm_id);
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
  if (!patch) return null;
  return (
    <GameDetailsView
      game={patchSummary({ ...patch, game_installed: false, patch_installed: false })}
      collection={collection}
      onBack={onBack}
      descriptionCredit={<TranslationCredit gameId={patch.scummvm_id} />}
      actions={
        <div className="space-y-2">
          <StoreInstallButtons
            installed={false}
            scummvmId={patch.scummvm_id}
            steamAppId={patch.steam_app_id}
            gogId={patch.gog_id}
          />
          <TranslationContributionButton gameId={patch.scummvm_id} />
        </div>
      }
    />
  );
}
