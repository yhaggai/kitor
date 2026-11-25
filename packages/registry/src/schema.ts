import { z } from "zod";

import {
  BundledAdapter,
  DetectionCheckKind,
  DetectionDigest,
  DetectionEdition,
  DetectionMode,
  DetectionPathStyle,
  InstallTarget,
  LaunchMethod,
  LaunchPreparation,
  LaunchRoot,
  LaunchWorkingDirectory,
  NativePreparation,
  PatchLanguage,
  Platform,
  SourceType,
  Store,
  Tag,
  TextNormalization,
} from "./index";

export const PlatformSchema = z.enum(Platform);
export const TagSchema = z.enum(Tag);
export const StoreSchema = z.enum(Store);
export const SourceTypeSchema = z.enum(SourceType);
export const LanguageSchema = z.enum(PatchLanguage);
export const DetectionModeSchema = z.enum(DetectionMode);
export const DetectionDigestSchema = z.enum(DetectionDigest);
export const TextNormalizationSchema = z.enum(TextNormalization);
export const DetectionEditionSchema = z.enum(DetectionEdition);
export const LaunchMethodSchema = z.enum(LaunchMethod);
export const LaunchRootSchema = z.enum(LaunchRoot);
export const LaunchWorkingDirectorySchema = z.enum(LaunchWorkingDirectory);
export const NativePreparationSchema = z.enum(NativePreparation);
export const BundledAdapterSchema = z.enum(BundledAdapter);
export const InstallTargetSchema = z.enum(InstallTarget);
export const LaunchPreparationSchema = z.enum(LaunchPreparation);

const text = z.string().min(1);
const strings = z.array(text);
const uint = z.number().int().nonnegative();
const hash = z.string().regex(/^[a-fA-F0-9]{64}$/);
const date = z.iso.date();
const nullableText = text.nullish();
const relativePath = text.regex(/^(?!.*(?:^|\/)\.{1,2}(?:\/|$))[^/\\:]+(?:\/[^/\\:]+)*$/);
export const CommandSchema = z.strictObject({
  program: relativePath,
  method: LaunchMethodSchema.optional(),
  root: LaunchRootSchema.optional(),
  bundle_name: relativePath.optional(),
  working_directory: LaunchWorkingDirectorySchema.optional(),
  executable_files: z.array(relativePath).optional(),
  required_directories: z.array(relativePath).optional(),
  signatures: z
    .array(
      z.strictObject({
        path: relativePath,
        prefix: z.array(z.number().int().min(0).max(255)).min(1).max(4096),
      })
    )
    .optional(),
  preparation: NativePreparationSchema.optional(),
  args: z.array(z.string()).optional(),
  env: z.record(text, z.string()).optional(),
  required_files: z.array(relativePath).optional(),
});
export const LaunchProfileSchema = z.strictObject({
  validate_on_import: z.boolean().optional(),
  commands: z.partialRecord(PlatformSchema, CommandSchema).optional(),
  subdirectory: nullableText,
  args: z.array(z.string()).optional(),
  runtime: nullableText,
  runtime_data: nullableText,
  preparation: LaunchPreparationSchema.nullish(),
});
export const DetectionFileCheckSchema = z.discriminatedUnion("kind", [
  z.strictObject({
    kind: z.literal(DetectionCheckKind.Text),
    path: relativePath,
    equals: z.string().max(1048576),
    normalization: TextNormalizationSchema.optional(),
  }),
  z.strictObject({
    kind: z.literal(DetectionCheckKind.Prefix),
    path: relativePath,
    bytes: z.array(z.number().int().min(0).max(255)).min(1).max(4096),
    min_size: uint.optional(),
  }),
  z.strictObject({
    kind: z.literal(DetectionCheckKind.Digest),
    path: relativePath,
    algorithm: DetectionDigestSchema,
    bytes: uint.positive().optional(),
    size: uint.optional(),
    values: z.array(z.string().regex(/^(?:[a-fA-F0-9]{32}|[a-fA-F0-9]{64})$/)).min(1),
  }),
  z.strictObject({ kind: z.literal(DetectionCheckKind.Size), path: relativePath, min: uint }),
]);
const layoutPath = z.union([z.literal(""), z.literal("."), relativePath]);
export const DetectionLayoutSchema = z.strictObject({
  root: layoutPath,
  path_style: z.enum(DetectionPathStyle).optional(),
  ancestor: uint.max(3).optional(),
  selected_name: relativePath.optional(),
  data: layoutPath.optional(),
  edition: DetectionEditionSchema.optional(),
  required_files: z.array(relativePath).min(1),
  required_directories: z.array(relativePath).optional(),
  file_checks: z.array(DetectionFileCheckSchema).optional(),
});
export const BehaviorSchema = z.strictObject({
  detection: z
    .strictObject({
      mode: DetectionModeSchema.optional(),
      priority: z.number().int().optional(),
      layouts: z.array(DetectionLayoutSchema).optional(),
      extra_subdirectories: strings.optional(),
    })
    .optional(),
  bundled: BundledAdapterSchema.nullish(),
  bundled_resource: nullableText,
  target: InstallTargetSchema.optional(),
  launch: z
    .strictObject({
      original: LaunchProfileSchema,
      original_macos: LaunchProfileSchema.nullish(),
      patched: LaunchProfileSchema,
    })
    .nullish(),
  library_editions: strings.optional(),
  health_files: strings.optional(),
  manual_patch_manifest: nullableText,
});
export const SupportedBuildSchema = z.strictObject({
  store: StoreSchema,
  platform: PlatformSchema,
  build_id: nullableText,
  depot_id: nullableText,
  manifest_id: nullableText,
  evidence: text,
  released_on: date.nullish(),
  first_seen_on: date.nullish(),
  release_date_source: z.url().nullish(),
});
const VerifiedMismatchSchema = z.strictObject({
  store: StoreSchema,
  platform: PlatformSchema,
  depot_id: text,
  checked_manifest_id: text,
  evidence: text,
  paths: z.array(relativePath).min(1),
});
export const PatchSchema = z.strictObject({
  verified_mismatches: z.array(VerifiedMismatchSchema).optional(),
  supported_builds: z.array(SupportedBuildSchema).optional(),
  match_files: z.record(text, hash).optional(),
  release_revision: uint.optional(),
  install_manifest: nullableText,
  language: LanguageSchema,
  version: text,
  author: text,
  download_url: z.url(),
  source_type: SourceTypeSchema,
  checksum: hash.nullish(),
  file_size: uint.nullish(),
  description: z.string(),
  filename: nullableText,
  post_install_script: z.partialRecord(PlatformSchema, nullableText).nullish(),
});
export const GameSchema = z.strictObject({
  scummvm_id: text,
  added_on: date.nullish(),
  steam_app_id: uint.max(4294967295).nullish(),
  title: text,
  title_he: nullableText,
  tags: z.array(TagSchema).optional(),
  translation_beta: z.boolean().nullish(),
  cover_url: z.url().nullish(),
  platforms: z.partialRecord(PlatformSchema, z.boolean()).nullish(),
  gog: z
    .strictObject({
      id: uint,
      slug: nullableText,
      title: nullableText,
      installers: z.array(z.strictObject({ id: z.string(), name: text, path: z.url() })).nullish(),
    })
    .nullish(),
  add_to_cart_url: z.url().nullish(),
  detection_patterns: z.strictObject({
    unique_files: strings,
    file_patterns: strings,
    detection_subdirectory: nullableText,
  }),
  patches: z.array(PatchSchema),
  discover_without_patch: z.boolean().optional(),
  behavior: BehaviorSchema.optional(),
  launch_config: z
    .strictObject({
      launcher: nullableText,
      subdirectory: nullableText,
      args: z.array(z.string()).optional(),
    })
    .nullish(),
  purchase_guide: nullableText,
});
export const RegistrySchema = z.strictObject({
  games: z.array(GameSchema),
  metadata: z.strictObject({ version: text, last_updated: date, source: text }),
});
export type Registry = z.infer<typeof RegistrySchema>;
export type RegistryGame = z.infer<typeof GameSchema>;
export type RegistryPatch = z.infer<typeof PatchSchema>;
export type { Platform, Tag } from "./index";

// Keep cross-record checks separate so the structural schema can also be exported to JSON Schema.
export function validateRegistry(input: unknown): Registry {
  const registry = RegistrySchema.parse(input);
  const ids = new Set<string>();
  for (const game of registry.games) {
    if (ids.has(game.scummvm_id)) throw new Error(`Duplicate game ID: ${game.scummvm_id}`);
    ids.add(game.scummvm_id);
    const behavior = game.behavior;
    const detection = behavior?.detection;
    if (detection?.mode === DetectionMode.Layouts && !detection.layouts?.length)
      throw new Error("Layout detection requires rules");
    for (const layout of detection?.layouts ?? []) {
      if (layout.selected_name?.includes("/"))
        throw new Error("Selected name must be a single directory name");
      for (const check of layout.file_checks ?? []) {
        if (
          check.kind === DetectionCheckKind.Digest &&
          check.values.some(
            (value) => value.length !== (check.algorithm === DetectionDigest.Sha256 ? 64 : 32)
          )
        )
          throw new Error("Detection digest length does not match algorithm");
      }
    }
    if (behavior?.bundled && behavior.bundled_resource) {
      throw new Error(`${game.scummvm_id}: choose bundled adapter or resource`);
    }
    for (const profile of Object.values(behavior?.launch ?? {})) {
      if (profile?.validate_on_import && !Object.keys(profile.commands ?? {}).length) {
        throw new Error("Import validation requires native commands");
      }
      for (const platform of PlatformSchema.options) {
        const command = profile?.commands?.[platform];
        if (command) validateCommand(command, platform);
      }
    }
  }
  return registry;
}

export const registryJsonSchema = () => z.toJSONSchema(RegistrySchema);

export type NativeCommand = z.infer<typeof CommandSchema>;
export function validateCommand(command: NativeCommand, platform: Platform): void {
  if (command.root === LaunchRoot.AppBundle && platform !== Platform.MacOS)
    throw new Error("App-bundle roots require macOS");
  if (
    command.preparation === NativePreparation.GrimHebrewPreferences &&
    platform !== Platform.Windows
  )
    throw new Error("Grim Fandango preference preparation requires Windows");
  const bundle = command.root === LaunchRoot.AppBundle;
  if (command.method === LaunchMethod.AppBundle && Object.keys(command.env ?? {}).length)
    throw new Error("Use executable launch when environment variables are required");
  if (bundle !== Boolean(command.bundle_name))
    throw new Error("App-bundle roots require a bundle name; game roots cannot specify one");
  if (
    command.bundle_name &&
    (command.bundle_name.includes("/") || !command.bundle_name.toLowerCase().endsWith(".app"))
  )
    throw new Error("Bundle name must be a single .app directory name");
  if (command.method === LaunchMethod.AppBundle && !bundle)
    throw new Error("App-bundle launch requires an app-bundle root");
  if (
    command.working_directory &&
    command.working_directory !== LaunchWorkingDirectory.Root &&
    !bundle
  )
    throw new Error("Bundle working directories require an app-bundle root");
}

export type LaunchProfile = z.infer<typeof LaunchProfileSchema>;

export type DetectionLayout = z.infer<typeof DetectionLayoutSchema>;
export type DetectionFileCheck = z.infer<typeof DetectionFileCheckSchema>;
