import { create } from "zustand";

type InstalledPatchInfo = {
  id: number;
  scummvm_id: string;
  language: string;
  version: string;
  installed_date: string;
  status: string;
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
};

type DownloadProgress = {
  downloaded: number;
  total: number;
  percentage: number;
};

type AppStore = {
  games: Game[];
  patches: Patch[];
  selectedGame: Game | null;
  downloadProgress: DownloadProgress | null;
  isScummVMInstalled: boolean;

  setGames: (games: Game[]) => void;
  setPatches: (patches: Patch[]) => void;
  setSelectedGame: (game: Game | null) => void;
  setDownloadProgress: (progress: DownloadProgress | null) => void;
  setScummVMInstalled: (installed: boolean) => void;
};

export const useAppStore = create<AppStore>((set) => ({
  games: [],
  patches: [],
  selectedGame: null,
  downloadProgress: null,
  isScummVMInstalled: false,

  setGames: (games) => set({ games }),
  setPatches: (patches) => set({ patches }),
  setSelectedGame: (game) => set({ selectedGame: game }),
  setDownloadProgress: (progress) => set({ downloadProgress: progress }),
  setScummVMInstalled: (installed) => set({ isScummVMInstalled: installed }),
}));

export type { Game, Patch, DownloadProgress, InstalledPatchInfo, AvailablePatchInfo };
