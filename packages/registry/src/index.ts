/** Shared registry vocabulary. Wire values remain compatible with the Rust API. */
export const Platform = {
  Windows: "windows",
  MacOS: "osx",
  Linux: "linux",
} as const;
export type Platform = (typeof Platform)[keyof typeof Platform];

export const Tag = {
  Classic: "classic",
  Comedy: "comedy",
  Detective: "detective",
  Fantasy: "fantasy",
  Horror: "horror",
  HotKitor: "hot-kitor",
  Indie: "indie",
  Kids: "kids",
  Puzzle: "puzzle",
  SciFi: "sci-fi",
  Story: "story",
  Thriller: "thriller",
} as const;
export type Tag = (typeof Tag)[keyof typeof Tag];

export const Store = {
  Steam: "steam",
  Gog: "gog",
} as const;
export type Store = (typeof Store)[keyof typeof Store];

export const SourceType = {
  GitHub: "github",
  Bundled: "bundled",
  LocalPreview: "local-preview",
} as const;
export type SourceType = (typeof SourceType)[keyof typeof SourceType];

export const PatchLanguage = {
  Hebrew: "he",
  HebrewNiqqud: "he-x-nikud",
} as const;
export type PatchLanguage = (typeof PatchLanguage)[keyof typeof PatchLanguage];

export const DetectionMode = { Patterns: "patterns", Layouts: "layouts" } as const;
export type DetectionMode = (typeof DetectionMode)[keyof typeof DetectionMode];
export const DetectionPathStyle = { Canonical: "canonical", Preserve: "preserve" } as const;
export type DetectionPathStyle = (typeof DetectionPathStyle)[keyof typeof DetectionPathStyle];
export const DetectionCheckKind = {
  Text: "text",
  Prefix: "prefix",
  Digest: "digest",
  Size: "size",
} as const;
export type DetectionCheckKind = (typeof DetectionCheckKind)[keyof typeof DetectionCheckKind];
export const DetectionDigest = { Sha256: "sha256", Md5: "md5" } as const;
export type DetectionDigest = (typeof DetectionDigest)[keyof typeof DetectionDigest];
export const TextNormalization = { Exact: "exact", TrimCrLf: "trim_crlf" } as const;
export type TextNormalization = (typeof TextNormalization)[keyof typeof TextNormalization];
export const DetectionEdition = { Classic: "classic", Remastered: "remastered" } as const;
export type DetectionEdition = (typeof DetectionEdition)[keyof typeof DetectionEdition];

export const LaunchMethod = { Executable: "executable", AppBundle: "app_bundle" } as const;
export type LaunchMethod = (typeof LaunchMethod)[keyof typeof LaunchMethod];
export const LaunchRoot = { GameDirectory: "game_directory", AppBundle: "app_bundle" } as const;
export type LaunchRoot = (typeof LaunchRoot)[keyof typeof LaunchRoot];
export const LaunchWorkingDirectory = {
  Root: "root",
  BundleParent: "bundle_parent",
  BundleResources: "bundle_resources",
} as const;
export type LaunchWorkingDirectory =
  (typeof LaunchWorkingDirectory)[keyof typeof LaunchWorkingDirectory];
export const NativePreparation = { GrimHebrewPreferences: "grim_hebrew_preferences" } as const;
export type NativePreparation = (typeof NativePreparation)[keyof typeof NativePreparation];

export const BundledAdapter = {
  Tentacle: "tentacle",
} as const;
export type BundledAdapter = (typeof BundledAdapter)[keyof typeof BundledAdapter];

export const InstallTarget = {
  GameDirectory: "game_directory",
  DrifterUserData: "drifter_user_data",
} as const;
export type InstallTarget = (typeof InstallTarget)[keyof typeof InstallTarget];

export const LaunchPreparation = {
  KyrandiaSignature: "kyrandia_signature",
  TentacleClassic: "tentacle_classic",
} as const;
export type LaunchPreparation = (typeof LaunchPreparation)[keyof typeof LaunchPreparation];

export type Platforms = Partial<Record<Platform, boolean>>;
// Store-only discovery entries have no patch language or download source.
export const CatalogSourceType = { ...SourceType, Store: "store" } as const;
export type CatalogSourceType = (typeof CatalogSourceType)[keyof typeof CatalogSourceType];
