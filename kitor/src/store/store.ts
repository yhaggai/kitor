import { create } from "zustand";

type InstalledPatchInfo = {
  id: number;
  scummvm_id: string;
  language: string;
  version: string;
  installed_date: string;
  status: string;
  backup_path?: string;
};

type AvailablePatchInfo = {
  language: string;
  version: string;
  author: string;
  description: string;
};

type Game = {
  id: number;
  scummvm_id: string;
  title: string;
  install_path: string;
  cover_url?: string;
  detected_version?: string;
  added_date: string;
  last_played?: string;
  has_available_patch?: boolean;
  installed_patches?: InstalledPatchInfo[];
  available_patches?: AvailablePatchInfo[];
};

type Patch = {
  scummvm_id: string;
  game_title: string;
  language: string;
  version: string;
  download_url: string;
  source_type: string;
  file_size?: number;
  checksum?: string;
  author: string;
  description: string;
  filename?: string;
  post_install_script?: string;
};

type DownloadProgress = {
  downloaded: number;
  total: number;
  percentage: number;
};

type ScummVMDownloadProgress = {
  progress: number;
  step: string;
};

type AppStore = {
  games: Game[];
  patches: Patch[];
  selectedGame: Game | null;
  downloadProgress: DownloadProgress | null;
  isScummVMReady: boolean;
  scummvmDownloadInProgress: boolean;
  scummvmDownloadProgress: ScummVMDownloadProgress | null;

  setGames: (games: Game[]) => void;
  setPatches: (patches: Patch[]) => void;
  setSelectedGame: (game: Game | null) => void;
  setDownloadProgress: (progress: DownloadProgress | null) => void;
  setScummVMReady: (ready: boolean) => void;
  setScummVMDownloadInProgress: (inProgress: boolean) => void;
  setScummVMDownloadProgress: (progress: ScummVMDownloadProgress | null) => void;
};

export const useAppStore = create<AppStore>((set) => ({
  games: [],
  patches: [],
  selectedGame: null,
  downloadProgress: null,
  isScummVMReady: false,
  scummvmDownloadInProgress: false,
  scummvmDownloadProgress: null,

  setGames: (games) => set({ games }),
  setPatches: (patches) => set({ patches }),
  setSelectedGame: (game) => set({ selectedGame: game }),
  setDownloadProgress: (progress) => set({ downloadProgress: progress }),
  setScummVMReady: (ready) => set({ isScummVMReady: ready }),
  setScummVMDownloadInProgress: (inProgress) => set({ scummvmDownloadInProgress: inProgress }),
  setScummVMDownloadProgress: (progress) => set({ scummvmDownloadProgress: progress }),
}));

export type {
  Game,
  Patch,
  DownloadProgress,
  InstalledPatchInfo,
  AvailablePatchInfo,
  ScummVMDownloadProgress,
};
