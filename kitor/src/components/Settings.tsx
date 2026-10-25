import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { invoke } from "@tauri-apps/api/core";
import {
  AlertCircle,
  CheckCircle,
  Download,
  ExternalLink,
  FolderOpen,
  MoreVertical,
  RefreshCw,
  Trash2,
  XCircle,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import SteamAuthCard from "@/components/SteamAuthCard";
import GogAuthCard from "@/components/GogAuthCard";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { useDialog } from "@/hooks/useDialog";
import { useSettingsState } from "@/hooks/useSettingsState";
import type { Game } from "@/store/store";
import { useAppStore } from "@/store/store";

type ScummVMStatus = {
  downloaded: boolean;
  path?: string;
  version?: string;
};

export default function Settings() {
  const {
    isScummVMReady,
    setScummVMReady,
    setGames,
    scummvmDownloadInProgress,
    setScummVMDownloadInProgress,
    scummvmDownloadProgress,
    setScummVMDownloadProgress,
    scummvmVersion,
    setScummVMVersion,
  } = useAppStore();
  const {
    isClearingCovers,
    isUninstalling,
    updateAvailable,
    setIsClearingCovers,
    setIsUninstalling,
    setUpdateAvailable,
  } = useSettingsState();
  const { alert, confirm } = useDialog();
  const [latestScummVMVersion, setLatestScummVMVersion] = useState<string>("");

  const fetchLatestVersion = useCallback(async () => {
    try {
      const version = await invoke<string>("get_latest_scummvm_version");
      setLatestScummVMVersion(version);
    } catch (error) {
      console.error("Failed to fetch latest ScummVM version:", error);
      // Set empty string on error so UI doesn't show stale data
      setLatestScummVMVersion("");
    }
  }, []);

  const checkScummVMStatus = useCallback(async () => {
    try {
      const status = (await invoke("check_scummvm_status")) as ScummVMStatus;
      setScummVMReady(status.downloaded);
      const version = status.version || "Unknown";
      setScummVMVersion(version);
    } catch (error) {
      console.error("Failed to check ScummVM status:", error);
      setScummVMReady(false);
      setScummVMVersion(null);
    }
  }, [setScummVMReady, setScummVMVersion]);

  useEffect(() => {
    fetchLatestVersion();
  }, [fetchLatestVersion]);

  useEffect(() => {
    if (scummvmVersion && latestScummVMVersion) {
      const isAvailable =
        scummvmVersion !== latestScummVMVersion &&
        scummvmVersion !== "Unknown" &&
        scummvmVersion !== "Missing/Corrupted";
      setUpdateAvailable(isAvailable);
    }
  }, [scummvmVersion, latestScummVMVersion, setUpdateAvailable]);

  const handleDownloadScummVM = async () => {
    try {
      setScummVMDownloadInProgress(true);
      await invoke("download_scummvm_binary");
      await checkScummVMStatus();
    } catch (error) {
      console.error("Failed to download ScummVM:", error);
      await alert({ title: "Download Failed", description: String(error) });
    } finally {
      setScummVMDownloadInProgress(false);
      setScummVMDownloadProgress(null);
    }
  };

  const handleUninstallScummVM = async () => {
    const confirmed = await confirm({
      title: "Uninstall ScummVM?",
      description: "Are you sure you want to uninstall ScummVM? You can download it again anytime.",
      confirmText: "Uninstall",
      cancelText: "Cancel",
    });
    if (!confirmed) {
      return;
    }

    try {
      setIsUninstalling(true);
      await invoke("uninstall_scummvm");
      await checkScummVMStatus();
    } catch (error) {
      console.error("Failed to uninstall ScummVM:", error);
      await alert({ title: "Uninstall Failed", description: String(error) });
    } finally {
      setIsUninstalling(false);
    }
  };

  const handleUpdateScummVM = async () => {
    const confirmed = await confirm({
      title: "Update ScummVM?",
      description: "This will download and install the latest version of ScummVM. Continue?",
      confirmText: "Update",
      cancelText: "Cancel",
    });
    if (!confirmed) {
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
      await alert({ title: "Update Failed", description: String(error) });
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
      await alert({ title: "Clear Cache Failed", description: String(error) });
    } finally {
      setIsClearingCovers(false);
    }
  };

  const handleOpenScummVM = async () => {
    try {
      await invoke("open_scummvm");
    } catch (error) {
      console.error("Failed to open ScummVM:", error);
      await alert({ title: "Open Failed", description: String(error) });
    }
  };

  const handleOpenScummVMFolder = async () => {
    try {
      await invoke("open_scummvm_folder");
    } catch (error) {
      console.error("Failed to open ScummVM folder:", error);
      await alert({ title: "Open Folder Failed", description: String(error) });
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

      <GogAuthCard />
      <SteamAuthCard />

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
              <Button onClick={handleOpenScummVM} variant="outline" className="flex-1">
                <ExternalLink className="h-4 w-4 mr-2" />
                Open ScummVM
              </Button>
              {updateAvailable && (
                <Button onClick={handleUpdateScummVM} variant="default">
                  <RefreshCw className="h-4 w-4 mr-2" />
                  Update to {latestScummVMVersion}
                </Button>
              )}
              <DropdownMenu.Root>
                <DropdownMenu.Trigger asChild>
                  <Button variant="outline" size="icon">
                    <MoreVertical className="h-4 w-4" />
                  </Button>
                </DropdownMenu.Trigger>
                <DropdownMenu.Portal>
                  <DropdownMenu.Content
                    className="min-w-[160px] rounded-md border bg-background p-1 shadow-lg"
                    align="end"
                  >
                    <DropdownMenu.Item
                      className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
                      onSelect={handleOpenScummVMFolder}
                    >
                      <FolderOpen className="mr-2 h-4 w-4" />
                      Open Folder
                    </DropdownMenu.Item>
                    <DropdownMenu.Separator className="my-1 h-px bg-border" />
                    <DropdownMenu.Item
                      className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-destructive outline-none hover:bg-accent focus:bg-accent"
                      onSelect={handleUninstallScummVM}
                      disabled={isUninstalling}
                    >
                      <Trash2 className="mr-2 h-4 w-4" />
                      Uninstall
                    </DropdownMenu.Item>
                  </DropdownMenu.Content>
                </DropdownMenu.Portal>
              </DropdownMenu.Root>
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
