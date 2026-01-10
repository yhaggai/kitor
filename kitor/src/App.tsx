import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef } from "react";
import GameLibrary from "@/components/GameLibrary";
import PatchBrowser from "@/components/PatchBrowser";
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
  const {
    setGames,
    setScummVMReady,
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
    } catch (error) {
      console.error("Failed to check ScummVM:", error);
      setScummVMReady(false);
    }
  }, [setScummVMReady]);

  useEffect(() => {
    loadGames();
    checkScummVM();

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
        <header className="mb-8">
          <h1 className="text-4xl font-bold mb-2">Kitor</h1>
          <p className="text-muted-foreground">ScummVM Game Library & Translation Patch Manager</p>
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
