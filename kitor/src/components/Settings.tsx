import { invoke } from "@tauri-apps/api/core";
import { AlertCircle, CheckCircle, XCircle } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import type { Game } from "@/store/store";
import { useAppStore } from "@/store/store";

export default function Settings() {
  const { isScummVMInstalled, setScummVMInstalled, setGames } = useAppStore();
  const [scummvmPath, setScummvmPath] = useState<string>("");
  const [isClearingCovers, setIsClearingCovers] = useState(false);

  const checkScummVMPath = useCallback(async () => {
    try {
      const path = (await invoke("get_scummvm_path")) as string;
      console.log("ScummVM found at:", path);
      setScummvmPath(path);
      setScummVMInstalled(true);
    } catch (error) {
      console.error("ScummVM not found:", error);
      setScummVMInstalled(false);
    }
  }, [setScummVMInstalled]);

  useEffect(() => {
    checkScummVMPath();
  }, [checkScummVMPath]);

  const handleClearCoverCache = async () => {
    try {
      setIsClearingCovers(true);
      await invoke("clear_cover_cache");
      const games = (await invoke("get_games_with_patches")) as Game[];
      setGames(games);
    } catch (error) {
      console.error("Failed to clear cover cache:", error);
      alert(`Failed to clear cover cache: ${error}`);
    } finally {
      setIsClearingCovers(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold mb-2">Settings</h2>
        <p className="text-muted-foreground">Configure Kitor and check system requirements</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Cover Images</CardTitle>
          <CardDescription>Clear cached covers to force re-download from registry</CardDescription>
        </CardHeader>
        <CardContent className="flex items-center gap-2">
          <Button variant="outline" onClick={handleClearCoverCache} disabled={isClearingCovers}>
            {isClearingCovers ? "Clearing..." : "Clear Cover Cache"}
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>ScummVM Installation</CardTitle>
          <CardDescription>ScummVM is required to run games</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-3">
            {isScummVMInstalled ? (
              <>
                <CheckCircle className="h-5 w-5 text-green-500" />
                <div className="flex-1">
                  <p className="font-medium">ScummVM is installed</p>
                  <p className="text-sm text-muted-foreground">{scummvmPath}</p>
                </div>
                <Badge variant="secondary">Ready</Badge>
              </>
            ) : (
              <>
                <XCircle className="h-5 w-5 text-destructive" />
                <div className="flex-1">
                  <p className="font-medium">ScummVM not found</p>
                  <p className="text-sm text-muted-foreground">
                    Please install ScummVM to launch games
                  </p>
                </div>
                <Badge variant="destructive">Required</Badge>
              </>
            )}
          </div>

          {!isScummVMInstalled && (
            <div className="bg-muted p-4 rounded-lg">
              <div className="flex gap-2 mb-2">
                <AlertCircle className="h-4 w-4 text-muted-foreground mt-0.5" />
                <div>
                  <p className="font-medium text-sm">How to install ScummVM:</p>
                  <ol className="text-sm text-muted-foreground list-decimal list-inside mt-2 space-y-1">
                    <li>
                      Download from{" "}
                      <a
                        href="https://www.scummvm.org/downloads/"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-primary hover:underline"
                      >
                        scummvm.org
                      </a>
                    </li>
                    <li>Run the installer</li>
                    <li>Restart Kitor</li>
                  </ol>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>About Kitor</CardTitle>
          <CardDescription>ScummVM Translation Patch Manager</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-muted-foreground space-y-2">
            <p>Version: 0.1.0</p>
            <p>
              Kitor helps you manage ScummVM games and easily install community translation patches.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
