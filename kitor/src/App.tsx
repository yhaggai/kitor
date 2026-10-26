import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef } from "react";
import BugReportDialog from "@/components/BugReportDialog";
import GameLibrary from "@/components/GameLibrary";
import PatchBrowser from "@/components/PatchBrowser";
import { useGogLibrary } from "@/store/gog";
import { useSteamStore } from "@/store/steam";
import Settings from "@/components/Settings";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
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
  useEffect(() => {
    const unlisten = listen("gog-login-success", () => {
      useGogLibrary.getState().clear();
      void useGogLibrary.getState().refresh();
    });
    void useGogLibrary.getState().refresh();
    return () => { void unlisten.then((stop) => stop()); };
  }, []);
  useEffect(() => { void useSteamStore.getState().run("get_steam_status"); }, []);
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
      await Promise.all([loadGames(), checkScummVM()]);
      // Close splash screen once app is ready
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
    <div className="min-h-screen bg-background">
      <div className="container mx-auto p-6">
        <header className="mb-8 flex flex-wrap items-center justify-between gap-4">
          <div>
            <h1 className="text-4xl font-bold mb-2">Kitor</h1>
            <p className="text-muted-foreground">ScummVM Game Library & Translation Patch Manager</p>
          </div>
          <BugReportDialog />
        </header>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid w-full grid-cols-3 mb-6">
            <TabsTrigger value="library">Library</TabsTrigger>
            <TabsTrigger value="patches">Patches</TabsTrigger>
            <TabsTrigger value="settings">Settings</TabsTrigger>
          </TabsList>

          <TabsContent value="library" className="space-y-4">
            <GameLibrary onRefresh={loadGames} />
          </TabsContent>

          <TabsContent value="patches" className="space-y-4">
            <PatchBrowser />
          </TabsContent>

          <TabsContent value="settings" className="space-y-4">
            <Settings />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

export default App;
