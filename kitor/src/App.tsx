import { AppShell } from "@kitor/ui";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import AppUpdateDialog from "@/components/AppUpdateDialog";
import BugReportDialog from "@/components/BugReportDialog";
import ContributorAccount from "@/components/ContributorAccount";
import GameLibrary from "@/components/GameLibrary";
import PatchBrowser from "@/components/PatchBrowser";
import Settings from "@/components/Settings";
import TranslationRequestDialog from "@/components/TranslationRequestDialog";

import { appUpdateStore } from "@/store/appUpdates";
import { useGogLibrary } from "@/store/gog";
import { useSteamStore } from "@/store/steam";
import type { Game, ScummVMDownloadProgress } from "@/store/store";
import { useAppStore } from "@/store/store";

type DownloadProgress = {
  percentage: number;
  downloaded: number;
  total: number;
};

type ScummVMStatus = {
  downloaded: boolean;
  path?: string;
  version?: string;
};

function App() {
  const [navigationResetKey, setNavigationResetKey] = useState(0);
  useEffect(() => {
    void appUpdateStore.getState().check();
  }, []);
  useEffect(() => {
    const unlisten = listen("gog-login-success", () => {
      useGogLibrary.getState().clear();
      void useGogLibrary.getState().refresh();
    });
    void useGogLibrary.getState().refresh();
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);
  useEffect(() => {
    void useSteamStore.getState().run("get_steam_status");
  }, []);
  const {
    setGames,
    setScummVMReady,
    setScummVMVersion,
    setDownloadProgress,
    setScummVMDownloadProgress,
    scummvmDownloadInProgress,
    activeTab,
    setActiveTab,
  } = useAppStore();

  // Track download state for event listener (ref avoids stale closure)
  const isDownloadingRef = useRef(scummvmDownloadInProgress);
  useEffect(() => {
    isDownloadingRef.current = scummvmDownloadInProgress;
  }, [scummvmDownloadInProgress]);

  const loadGames = useCallback(async () => {
    try {
      const games = await invoke("get_games_with_patches");
      setGames(games as Game[]);
    } catch (error) {
      console.error("Failed to load games:", error);
    }
  }, [setGames]);

  const checkScummVM = useCallback(async () => {
    try {
      const status = (await invoke("check_scummvm_status")) as ScummVMStatus;
      setScummVMReady(status.downloaded);
      setScummVMVersion(status.version || null);
    } catch (error) {
      console.error("Failed to check ScummVM:", error);
      setScummVMReady(false);
      setScummVMVersion(null);
    }
  }, [setScummVMReady, setScummVMVersion]);

  useEffect(() => {
    const initApp = async () => {
      const minimumSplashDuration = new Promise<void>((resolve) => {
        window.setTimeout(resolve, 3000);
      });
      await Promise.all([loadGames(), checkScummVM()]);
      // Discovery is independent of installed records and never blocks offline use.
      void invoke("refresh_patch_registry")
        .then(loadGames)
        .catch((error) => {
          console.info("Translation update check unavailable; using cached catalog:", error);
        });
      // Close only after initialization and the minimum display time finish.
      await minimumSplashDuration;
      try {
        await invoke("close_splashscreen");
      } catch (error) {
        console.error("Failed to close splash screen:", error);
      }
    };

    initApp();

    const unlistenDownload = listen("download-progress", (event) => {
      setDownloadProgress(event.payload as DownloadProgress);
    });

    const unlistenScummVM = listen("scummvm-download-progress", (event) => {
      if (isDownloadingRef.current) {
        setScummVMDownloadProgress(event.payload as ScummVMDownloadProgress);
      }
    });

    return () => {
      unlistenDownload.then((fn) => fn());
      unlistenScummVM.then((fn) => fn());
    };
  }, [loadGames, checkScummVM, setDownloadProgress, setScummVMDownloadProgress]);

  return (
    <AppShell
      account={<ContributorAccount />}
      active={
        activeTab === "patches" ? "discovery" : activeTab === "settings" ? "settings" : "library"
      }
      onNavigate={(view) => {
        setNavigationResetKey((key) => key + 1);
        setActiveTab(view === "discovery" ? "patches" : view);
        window.scrollTo({ top: 0, behavior: "instant" });
      }}
      footer={
        <>
          <BugReportDialog />
          <TranslationRequestDialog />
        </>
      }
    >
      <AppUpdateDialog />
      <div hidden={activeTab !== "library"}>
        <GameLibrary onRefresh={loadGames} navigationResetKey={navigationResetKey} />
      </div>
      <div hidden={activeTab !== "patches"}>
        <PatchBrowser navigationResetKey={navigationResetKey} />
      </div>
      {activeTab === "settings" && <Settings />}
    </AppShell>
  );
}

export default App;
