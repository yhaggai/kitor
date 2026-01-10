import { invoke } from "@tauri-apps/api/core";
import { Download } from "lucide-react";
import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { useDialog } from "@/hooks/useDialog";
import { type Patch, useAppStore } from "@/store/store";

export default function PatchBrowser() {
  const [patches, setPatches] = useState<Patch[]>([]);
  const [loading, setLoading] = useState(false);
  const [installing, setInstalling] = useState<string | null>(null);
  const { downloadProgress } = useAppStore();
  const { alert } = useDialog();

  useEffect(() => {
    async function loadPatches() {
      try {
        setLoading(true);
        const allPatches = (await invoke("get_all_patches")) as Patch[];
        setPatches(allPatches);
      } catch (error) {
        console.error("Failed to load patches:", error);
      } finally {
        setLoading(false);
      }
    }
    loadPatches();
  }, []);

  const handleInstall = async (patch: Patch) => {
    try {
      setInstalling(`${patch.scummvm_id}-${patch.language}-${patch.version}`);
      await invoke("download_and_install_patch", {
        scummvmId: patch.scummvm_id,
        language: patch.language,
        version: patch.version,
      });
      await alert({ title: "Success", description: "Patch installed successfully!" });
    } catch (error) {
      console.error("Failed to install patch:", error);
      await alert({ title: "Install Failed", description: String(error) });
    } finally {
      setInstalling(null);
    }
  };

  if (loading) {
    return <div className="text-center py-12">Loading patches...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold mb-2">Available Patches</h2>
        <p className="text-muted-foreground">Browse and install community translation patches</p>
      </div>

      {patches.length === 0 ? (
        <Card>
          <CardContent className="pt-6 text-center">
            <p className="text-muted-foreground">
              No patches available. Add patches manually or configure a patch registry.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4">
          {patches.map((patch) => (
            <Card key={`${patch.scummvm_id}-${patch.language}-${patch.version}`}>
              <CardHeader>
                <div className="flex justify-between items-start">
                  <div>
                    <CardTitle>{patch.game_title}</CardTitle>
                    <CardDescription>
                      {patch.language} translation - v{patch.version}
                    </CardDescription>
                  </div>
                  <Badge>{patch.source_type}</Badge>
                </div>
              </CardHeader>
              <CardContent>
                {patch.description && (
                  <p className="text-sm text-muted-foreground mb-2">{patch.description}</p>
                )}
                {patch.author && <p className="text-xs text-muted-foreground">By {patch.author}</p>}
              </CardContent>
              <CardFooter>
                <Button
                  onClick={() => handleInstall(patch)}
                  disabled={installing !== null}
                  className="w-full"
                >
                  {installing === `${patch.scummvm_id}-${patch.language}-${patch.version}` ? (
                    <>
                      <Download className="mr-2 h-4 w-4 animate-spin" />
                      Installing... {downloadProgress?.percentage || 0}%
                    </>
                  ) : (
                    <>
                      <Download className="mr-2 h-4 w-4" />
                      Install Patch
                    </>
                  )}
                </Button>
              </CardFooter>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
