import { create } from "zustand";

type InstalledPatchInfo = {
  id: number;
  scummvm_id: string;
  version: string;
  installed_date: string;
  status: string;
  language: string;
  backup_path?: string;
};

type AvailablePatchInfo = {
  language: string;
  version: string;
  author: string;
  description: string;
};

export type GameHealth = {
  game_status: "present" | "missing" | "unrecognized";
  patch_status: "installed" | "not_installed" | "damaged" | "unverified";
  details: string[];
  checked_at: string;
};

type Game = {
  health?: GameHealth;
  id: number;
  scummvm_id: string;
  title: string;
  install_path: string;
  cover_url?: string;
  detected_version?: string;
  added_date: string;
  last_played?: string;
  requires_scummvm?: boolean;
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
  purchase_guide?: string;
  cover_url?: string;
  steam_app_id?: number | null;
  gog_id?: number | null;
  gog_installer_path?: string | null;
  platforms?: {
    windows: boolean;
    osx: boolean;
    linux: boolean;
  };
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

type GogAuthState = {
  isAuthenticated: boolean;
  userEmail?: string;
};

type GogGame = {
  id: number;
  title: string;
  cover_url?: string;
};

type AppStore = {
  games: Game[];
  patches: Patch[];
  selectedGame: Game | null;
  downloadProgress: DownloadProgress | null;
  isScummVMReady: boolean;
  scummvmDownloadInProgress: boolean;
  scummvmDownloadProgress: ScummVMDownloadProgress | null;
  scummvmVersion: string | null;
  activeTab: string;
  highlightedGameId: number | null;
  gogAuth: GogAuthState;
  gogGames: GogGame[];

  setGames: (games: Game[]) => void;
  setPatches: (patches: Patch[]) => void;
  setSelectedGame: (game: Game | null) => void;
  setDownloadProgress: (progress: DownloadProgress | null) => void;
  setScummVMReady: (ready: boolean) => void;
  setScummVMDownloadInProgress: (inProgress: boolean) => void;
  setScummVMDownloadProgress: (progress: ScummVMDownloadProgress | null) => void;
  setScummVMVersion: (version: string | null) => void;
  setActiveTab: (tab: string) => void;
  setHighlightedGameId: (id: number | null) => void;
  setGogAuth: (auth: GogAuthState) => void;
  setGogGames: (games: GogGame[]) => void;
};

export const useAppStore = create<AppStore>((set) => ({
  games: [],
  patches: [],
  selectedGame: null,
  downloadProgress: null,
  isScummVMReady: false,
  scummvmDownloadInProgress: false,
  scummvmDownloadProgress: null,
  scummvmVersion: null,
  activeTab: "library",
  highlightedGameId: null,
  gogAuth: { isAuthenticated: false },
  gogGames: [],

  setGames: (games) => set({ games }),
  setPatches: (patches) => set({ patches }),
  setSelectedGame: (game) => set({ selectedGame: game }),
  setDownloadProgress: (progress) => set({ downloadProgress: progress }),
  setScummVMReady: (ready) => set({ isScummVMReady: ready }),
  setScummVMDownloadInProgress: (inProgress) => set({ scummvmDownloadInProgress: inProgress }),
  setScummVMDownloadProgress: (progress) => set({ scummvmDownloadProgress: progress }),
  setScummVMVersion: (version) => set({ scummvmVersion: version }),
  setActiveTab: (tab) => set({ activeTab: tab }),
  setHighlightedGameId: (id) => set({ highlightedGameId: id }),
  setGogAuth: (auth) => set({ gogAuth: auth }),
  setGogGames: (games) => set({ gogGames: games }),
}));

export type PatchWithStatus = Patch & {
  game_installed: boolean;
  patch_installed: boolean;
  game_id?: number;
  cover_url?: string;
  purchase_guide?: string;
  steam_app_id?: number | null;
  gog_id?: number | null;
  gog_installer_path?: string | null;
  platforms?: {
    windows: boolean;
    osx: boolean;
    linux: boolean;
  };
};

export type {
  Game,
  Patch,
  DownloadProgress,
  InstalledPatchInfo,
  AvailablePatchInfo,
  ScummVMDownloadProgress,
  GogAuthState,
  GogGame,
};
