import { ManageMenu, scummvmIcon, useTranslation } from "@kitor/ui";

import { invoke } from "@tauri-apps/api/core";
import {
  AlertCircle,
  CheckCircle,
  Download,
  ExternalLink,
  FolderOpen,
  RefreshCw,
  Trash2,
  XCircle,
} from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import AppUpdateCard from "@/components/AppUpdateCard";
import GogAuthCard from "@/components/GogAuthCard";
import SteamAuthCard from "@/components/SteamAuthCard";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { useDialog } from "@/hooks/useDialog";
import { useSettingsState } from "@/hooks/useSettingsState";
import { useAppStore } from "@/store/store";

type ScummVMStatus = {
  downloaded: boolean;
  path?: string;
  version?: string;
};

export default function Settings() {
  const { t, language, setLanguage } = useTranslation();
  const {
    isScummVMReady,
    setScummVMReady,
    scummvmDownloadInProgress,
    setScummVMDownloadInProgress,
    scummvmDownloadProgress,
    setScummVMDownloadProgress,
    scummvmVersion,
    setScummVMVersion,
  } = useAppStore();
  const { isUninstalling, updateAvailable, setIsUninstalling, setUpdateAvailable } =
    useSettingsState();
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
      await alert({ failure: true, title: t("Download Failed"), description: String(error) });
    } finally {
      setScummVMDownloadInProgress(false);
      setScummVMDownloadProgress(null);
    }
  };

  const handleUninstallScummVM = async () => {
    const confirmed = await confirm({
      title: t("Uninstall ScummVM?"),
      description: t(
        "Are you sure you want to uninstall ScummVM? You can download it again anytime."
      ),
      confirmText: t("Uninstall"),
      cancelText: t("Cancel"),
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
      await alert({ failure: true, title: t("Uninstall Failed"), description: String(error) });
    } finally {
      setIsUninstalling(false);
    }
  };

  const handleUpdateScummVM = async () => {
    const confirmed = await confirm({
      title: t("Update ScummVM?"),
      description: t("This will download and install the latest version of ScummVM. Continue?"),
      confirmText: t("Update"),
      cancelText: t("Cancel"),
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
      await alert({ failure: true, title: t("Update Failed"), description: String(error) });
    } finally {
      setIsUninstalling(false);
      setScummVMDownloadInProgress(false);
      setScummVMDownloadProgress(null);
    }
  };

  const handleOpenScummVM = async () => {
    try {
      await invoke("open_scummvm");
    } catch (error) {
      console.error("Failed to open ScummVM:", error);
      await alert({ failure: true, title: t("Open Failed"), description: String(error) });
    }
  };

  const handleOpenScummVMFolder = async () => {
    try {
      await invoke("open_scummvm_folder");
    } catch (error) {
      console.error("Failed to open ScummVM folder:", error);
      await alert({ failure: true, title: t("Open Folder Failed"), description: String(error) });
    }
  };

  return (
    <div className="kt-native-settings space-y-6">
      <div>
        <p className="kt-eyebrow">{t("MAKE YOURSELF AT HOME")}</p>
        <h1 className="text-4xl font-bold mb-2">{t("Settings")}</h1>
        <p className="text-muted-foreground">
          {t("Configure Kitor and check system requirements")}
        </p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>{t("Language")}</CardTitle>
          <CardDescription>{t("Choose the language used throughout Kitor.")}</CardDescription>
        </CardHeader>
        <CardContent>
          <label htmlFor="app-language" className="block mb-2">
            {t("App language")}
          </label>
          <select
            id="app-language"
            value={language}
            onChange={(event) => setLanguage(event.target.value === "he" ? "he" : "en")}
            className="rounded-md border border-border bg-background px-3 py-2 text-foreground"
          >
            <option value="en" lang="en">
              English
            </option>
            <option value="he" lang="he">
              עברית
            </option>
          </select>
        </CardContent>
      </Card>

      <AppUpdateCard />
      <GogAuthCard />
      <SteamAuthCard />

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-3">
            <img className="h-10 w-10 object-contain" src={scummvmIcon} alt="" />
            ScummVM
          </CardTitle>
          <CardDescription>
            {t("Kitor uses its own standalone ScummVM installation")}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-3">
            {isScummVMReady ? (
              <>
                <CheckCircle className="h-5 w-5 text-green-500" />
                <div className="flex-1">
                  <p className="font-medium">{t("ScummVM Ready")}</p>
                  <p className="text-sm text-muted-foreground">
                    {t("Version")} {scummvmVersion === "Unknown" ? t("Unknown") : scummvmVersion}
                  </p>
                </div>
                {updateAvailable && (
                  <Badge variant="default" className="bg-blue-600">
                    {t("Update Available")}{" "}
                  </Badge>
                )}
                {!updateAvailable && <Badge variant="secondary">{t("Ready")}</Badge>}
              </>
            ) : (
              <>
                <XCircle className="h-5 w-5 text-destructive" />
                <div className="flex-1">
                  <p className="font-medium">{t("ScummVM Not Downloaded")}</p>
                  <p className="text-sm text-muted-foreground">{t("Required to launch games")}</p>
                </div>
                <Badge variant="destructive">{t("Required")}</Badge>
              </>
            )}
          </div>

          {isScummVMReady && !scummvmDownloadInProgress && !isUninstalling && (
            <ManageMenu label={t("Manage ScummVM")}>
              <Button onClick={handleOpenScummVM} variant="outline">
                <ExternalLink size={18} />
                {t("Open ScummVM")}{" "}
              </Button>
              {updateAvailable && (
                <Button onClick={handleUpdateScummVM}>
                  <RefreshCw size={18} />
                  {t("Update to")} {latestScummVMVersion}
                </Button>
              )}
              <Button variant="outline" onClick={handleOpenScummVMFolder}>
                <FolderOpen size={18} />
                {t("Open Folder")}{" "}
              </Button>
              <Button variant="outline" onClick={handleUninstallScummVM} disabled={isUninstalling}>
                <Trash2 size={18} />
                {t("Uninstall")}{" "}
              </Button>
            </ManageMenu>
          )}

          {isUninstalling && (
            <div className="flex items-center justify-center py-2">
              <p className="text-sm text-muted-foreground">{t("Uninstalling...")}</p>
            </div>
          )}

          {!isScummVMReady && !scummvmDownloadInProgress && (
            <div className="bg-muted p-4 rounded-lg">
              <div className="flex gap-2 mb-3">
                <AlertCircle className="h-4 w-4 text-muted-foreground mt-0.5" />
                <div className="flex-1">
                  <p className="font-medium text-sm">{t("ScummVM Download Required")}</p>
                  <p className="text-sm text-muted-foreground mt-1">
                    {t(
                      "Kitor will download and manage its own ScummVM installation. This ensures compatibility and requires no manual setup."
                    )}{" "}
                  </p>
                </div>
              </div>
              <Button
                onClick={handleDownloadScummVM}
                disabled={scummvmDownloadInProgress}
                className="w-full"
              >
                <Download className="h-4 w-4 mr-2" />
                {t("Download ScummVM")}{" "}
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
          <CardTitle>{t("About Kitor")}</CardTitle>
          <CardDescription>{t("ScummVM Translation Patch Manager")}</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="text-sm text-muted-foreground space-y-2">
            <p>
              {t(
                "Kitor helps you manage ScummVM games and easily install community translation patches."
              )}{" "}
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
