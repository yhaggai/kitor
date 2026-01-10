import { invoke } from "@tauri-apps/api/core";
import { AlertCircle, CheckCircle, Download, RefreshCw, Trash2, XCircle } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import type { Game } from "@/store/store";
import { useAppStore } from "@/store/store";

type ScummVMStatus = {
  downloaded: boolean;
  path?: string;
  version?: string;
};

const LATEST_SCUMMVM_VERSION = "2.8.1"; // Should match backend constant

export default function Settings() {
  const {
    isScummVMReady,
    setScummVMReady,
    setGames,
    scummvmDownloadInProgress,
    setScummVMDownloadInProgress,
    scummvmDownloadProgress,
    setScummVMDownloadProgress,
  } = useAppStore();
  const [scummvmVersion, setScummvmVersion] = useState<string>("");
  const [isClearingCovers, setIsClearingCovers] = useState(false);
  const [isUninstalling, setIsUninstalling] = useState(false);
  const [updateAvailable, setUpdateAvailable] = useState(false);

  const checkScummVMStatus = useCallback(async () => {
    try {
      const status = (await invoke("check_scummvm_status")) as ScummVMStatus;
      setScummVMReady(status.downloaded);
      if (status.downloaded) {
        const version = status.version || "Unknown";
        setScummvmVersion(version);
        setUpdateAvailable(version !== LATEST_SCUMMVM_VERSION && version !== "Unknown");
      }
    } catch (error) {
      console.error("Failed to check ScummVM status:", error);
      setScummVMReady(false);
    }
  }, [setScummVMReady]);

  useEffect(() => {
    checkScummVMStatus();
  }, [checkScummVMStatus]);

  const handleDownloadScummVM = async () => {
    try {
      setScummVMDownloadInProgress(true);
      await invoke("download_scummvm_binary");
      await checkScummVMStatus();
    } catch (error) {
      console.error("Failed to download ScummVM:", error);
      alert(`Failed to download ScummVM: ${error}`);
    } finally {
      setScummVMDownloadInProgress(false);
      setScummVMDownloadProgress(null);
    }
  };

  const handleUninstallScummVM = async () => {
    if (
      !confirm("Are you sure you want to uninstall ScummVM? You can download it again anytime.")
    ) {
      return;
    }

    try {
      setIsUninstalling(true);
      await invoke("uninstall_scummvm");
      await checkScummVMStatus();
    } catch (error) {
      console.error("Failed to uninstall ScummVM:", error);
      alert(`Failed to uninstall ScummVM: ${error}`);
    } finally {
      setIsUninstalling(false);
    }
  };

  const handleUpdateScummVM = async () => {
    if (!confirm("This will download and install the latest version of ScummVM. Continue?")) {
      return;
    }

    try {
      // Uninstall current version first
      setIsUninstalling(true);
      await invoke("uninstall_scummvm");
      setIsUninstalling(false);

      // Download new version
      setScummVMDownloadInProgress(true);
      await invoke("download_scummvm_binary");
      await checkScummVMStatus();
    } catch (error) {
      console.error("Failed to update ScummVM:", error);
      alert(`Failed to update ScummVM: ${error}`);
    } finally {
      setIsUninstalling(false);
      setScummVMDownloadInProgress(false);
      setScummVMDownloadProgress(null);
    }
  };

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
          <CardTitle>ScummVM</CardTitle>
          <CardDescription>Kitor uses its own standalone ScummVM installation</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-3">
            {isScummVMReady ? (
              <>
                <CheckCircle className="h-5 w-5 text-green-500" />
                <div className="flex-1">
                  <p className="font-medium">ScummVM Ready</p>
                  <p className="text-sm text-muted-foreground">Version {scummvmVersion}</p>
                </div>
                {updateAvailable && (
                  <Badge variant="default" className="bg-blue-600">
                    Update Available
                  </Badge>
                )}
                {!updateAvailable && <Badge variant="secondary">Ready</Badge>}
              </>
            ) : (
              <>
                <XCircle className="h-5 w-5 text-destructive" />
                <div className="flex-1">
                  <p className="font-medium">ScummVM Not Downloaded</p>
                  <p className="text-sm text-muted-foreground">Required to launch games</p>
                </div>
                <Badge variant="destructive">Required</Badge>
              </>
            )}
          </div>

          {isScummVMReady && !scummvmDownloadInProgress && !isUninstalling && (
            <div className="flex gap-2">
              {updateAvailable && (
                <Button onClick={handleUpdateScummVM} variant="default" className="flex-1">
                  <RefreshCw className="h-4 w-4 mr-2" />
                  Update to {LATEST_SCUMMVM_VERSION}
                </Button>
              )}
              <Button
                onClick={handleUninstallScummVM}
                variant="destructive"
                disabled={isUninstalling}
                className={updateAvailable ? "" : "flex-1"}
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Uninstall
              </Button>
            </div>
          )}

          {isUninstalling && (
            <div className="flex items-center justify-center py-2">
              <p className="text-sm text-muted-foreground">Uninstalling...</p>
            </div>
          )}

          {!isScummVMReady && !scummvmDownloadInProgress && (
            <div className="bg-muted p-4 rounded-lg">
              <div className="flex gap-2 mb-3">
                <AlertCircle className="h-4 w-4 text-muted-foreground mt-0.5" />
                <div className="flex-1">
                  <p className="font-medium text-sm">ScummVM Download Required</p>
                  <p className="text-sm text-muted-foreground mt-1">
                    Kitor will download and manage its own ScummVM installation. This ensures
                    compatibility and requires no manual setup.
                  </p>
                </div>
              </div>
              <Button
                onClick={handleDownloadScummVM}
                disabled={scummvmDownloadInProgress}
                className="w-full"
              >
                <Download className="h-4 w-4 mr-2" />
                Download ScummVM
              </Button>
            </div>
          )}

          {scummvmDownloadInProgress && scummvmDownloadProgress && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">{scummvmDownloadProgress.step}</span>
                <span className="font-medium">{scummvmDownloadProgress.progress}%</span>
              </div>
              <Progress value={scummvmDownloadProgress.progress} />
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
