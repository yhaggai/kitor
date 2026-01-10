import * as Tooltip from "@radix-ui/react-tooltip";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { CheckCircle, Download, ExternalLink, Package, Play } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import AddGameDialog from "@/components/AddGameDialog";
import { PurchaseGuideDialog } from "@/components/PurchaseGuideDialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import { usePatchBrowserState } from "@/hooks/usePatchBrowserState";
import { type Game, type Patch, type PatchWithStatus, useAppStore } from "@/store/store";

export default function PatchBrowser() {
  const { state, actions } = usePatchBrowserState();
  const { patches, loading, showAddDialog, selectedGuide } = state;

  const { games, setActiveTab, setHighlightedGameId, setGames } = useAppStore();

  useEffect(() => {
    async function loadPatches() {
      try {
        actions.setLoading(true);
        const allPatches = (await invoke("get_all_patches")) as Patch[];

        // Map status to patches
        const patchesWithStatus = allPatches.map((patch) => {
          const game = games.find((g) => g.scummvm_id === patch.scummvm_id);
          const isGameInstalled = !!game;
          const isPatchInstalled =
            game?.installed_patches?.some(
              (p) => p.language === patch.language && p.version === patch.version
            ) ?? false;

          return {
            ...patch,
            game_installed: isGameInstalled,
            patch_installed: isPatchInstalled,
            game_id: game?.id,
            cover_url: game?.cover_url || patch.cover_url,
          };
        });

        actions.setPatches(patchesWithStatus);
      } catch (error) {
        console.error("Failed to load patches:", error);
      } finally {
        actions.setLoading(false);
      }
    }

    loadPatches();
  }, [games, actions]);

  const handleViewGame = (gameId: number) => {
    setActiveTab("library");
    setHighlightedGameId(gameId);
  };

  if (loading && patches.length === 0) {
    return <div className="text-center py-12">Loading patches...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold mb-2">Available Patches</h2>
        <p className="text-muted-foreground">Browse and install community translation patches</p>
      </div>

      <AddGameDialog
        open={showAddDialog}
        onOpenChange={(open) => (open ? actions.showAddDialog() : actions.hideAddDialog())}
        onAdded={async () => {
          const updatedGames = (await invoke("get_games_with_patches")) as Game[];
          setGames(updatedGames);
        }}
      />

      <PurchaseGuideDialog
        scummvmId={selectedGuide?.scummvmId || ""}
        gameTitle={selectedGuide?.title || ""}
        open={!!selectedGuide}
        onOpenChange={(open) => !open && actions.clearSelectedGuide()}
        onAddGame={() => actions.showAddDialog()}
      />

      {patches.length === 0 ? (
        <Card>
          <CardContent className="pt-6 text-center">
            <p className="text-muted-foreground">
              No patches available. Add patches manually or configure a patch registry.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
          {patches.map((patch) => (
            <PatchCard
              key={`${patch.scummvm_id}-${patch.language}-${patch.version}`}
              patch={patch}
              onViewGame={() => patch.game_id && handleViewGame(patch.game_id)}
              onShowGuide={() =>
                actions.setSelectedGuide({ scummvmId: patch.scummvm_id, title: patch.game_title })
              }
            />
          ))}
        </div>
      )}
    </div>
  );
}

type PatchCardProps = {
  patch: PatchWithStatus;
  onViewGame: () => void;
  onShowGuide: () => void;
};

function PatchCard({ patch, onViewGame, onShowGuide }: PatchCardProps) {
  const [coverError, setCoverError] = useState(false);

  const coverSrc = useMemo(() => {
    if (!patch.cover_url || coverError) return null;
    if (patch.cover_url.startsWith("http://") || patch.cover_url.startsWith("https://")) {
      return patch.cover_url;
    }
    const normalizedPath = patch.cover_url.split("\\").join("/");
    return convertFileSrc(normalizedPath);
  }, [coverError, patch.cover_url]);

  return (
    <Card className="hover:shadow-lg transition-shadow">
      <CardContent className="p-0 relative">
        <div className="aspect-[3/4] bg-gradient-to-br from-primary/20 to-secondary/20 rounded-t-lg flex items-center justify-center relative overflow-hidden">
          {coverSrc ? (
            <img
              src={coverSrc}
              alt={patch.game_title}
              className="w-full h-full object-cover rounded-t-lg"
              loading="lazy"
              onError={() => setCoverError(true)}
            />
          ) : (
            <Package className="h-16 w-16 text-muted-foreground" />
          )}

          <div className="absolute top-2 right-2 flex flex-col gap-1 items-end">
            <Badge variant="secondary" className="backdrop-blur-sm bg-background/80">
              {patch.language}
            </Badge>
            {patch.patch_installed && (
              <Badge variant="default" className="bg-green-600 backdrop-blur-sm">
                <CheckCircle className="w-3 h-3 mr-1" />
                Installed
              </Badge>
            )}
          </div>
        </div>
      </CardContent>
      <CardFooter className="flex flex-col gap-2 p-4">
        <div className="w-full">
          <Tooltip.Provider>
            <Tooltip.Root>
              <Tooltip.Trigger asChild>
                <h3 className="font-semibold text-sm truncate mb-1">{patch.game_title}</h3>
              </Tooltip.Trigger>
              <Tooltip.Portal>
                <Tooltip.Content
                  className="rounded-md bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md"
                  sideOffset={5}
                >
                  {patch.game_title}
                </Tooltip.Content>
              </Tooltip.Portal>
            </Tooltip.Root>
          </Tooltip.Provider>
          <p className="text-xs text-muted-foreground truncate">
            v{patch.version} • {patch.author}
          </p>
        </div>

        {patch.patch_installed ? (
          <Button className="w-full" size="sm" onClick={onViewGame}>
            <Play className="mr-2 h-3 w-3" />
            View Game
          </Button>
        ) : patch.game_installed ? (
          <Button className="w-full" size="sm" variant="default" onClick={onViewGame}>
            <Download className="mr-2 h-3 w-3" />
            Install Patch
          </Button>
        ) : (
          <Button className="w-full" size="sm" variant="secondary" onClick={onShowGuide}>
            <ExternalLink className="mr-2 h-3 w-3" />
            Purchase Guide
          </Button>
        )}
      </CardFooter>
    </Card>
  );
}
