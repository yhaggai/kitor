import type { CatalogSourceType, PatchLanguage, Platforms, Tag } from "@kitor/registry";
import type { RegistryPatch } from "@kitor/registry/schema";
import { create } from "zustand";

type InstalledPatchInfo = {
  release_revision: number;
  id: number;
  scummvm_id: string;
  version: string;
  installed_date: string;
  status: string;
  language: PatchLanguage | null;
  backup_path?: string;
};

type AvailablePatchInfo = Pick<RegistryPatch, "language" | "version" | "author" | "description"> & {
  release_revision: number;
};

export type GameHealth = {
  game_status: "present" | "missing" | "unrecognized";
  patch_status: "installed" | "not_installed" | "damaged" | "unverified";
  details: string[];
  checked_at: string;
};

type Game = {
  health?: GameHealth | null;
  id: number;
  tags?: Tag[];
  translation_beta?: boolean;
  added_on?: string;
  scummvm_id: string;
  title: string;
  title_he?: string | null;
  install_path: string;
  cover_url?: string;
  detected_version?: string;
  added_date: string;
  last_played?: string;
  requires_scummvm?: boolean;
  is_patched?: boolean;
  has_available_patch?: boolean;
  installed_patches?: InstalledPatchInfo[];
  available_patches?: AvailablePatchInfo[];
};

type Patch = Pick<
  RegistryPatch,
  "version" | "download_url" | "author" | "description" | "post_install_script"
> & {
  scummvm_id: string;
  game_title: string;
  title_he?: string | null;
  language: PatchLanguage | "";
  source_type: CatalogSourceType;
  file_size?: number;
  checksum?: string;
  filename?: string;
  purchase_guide?: string;
  cover_url?: string;
  steam_app_id?: number | null;
  gog_id?: number | null;
  gog_installer_path?: string | null;
  add_to_cart_url?: string | null;
  platforms?: Platforms;
  tags?: Tag[];
  translation_beta?: boolean;
  added_on?: string;
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
  activeTab: "patches",
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
};

export type {
  AvailablePatchInfo,
  DownloadProgress,
  Game,
  GogAuthState,
  GogGame,
  InstalledPatchInfo,
  Patch,
  ScummVMDownloadProgress,
};
