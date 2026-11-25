// Compile-time contract tests. Run with bun run registry:typecheck.

import type { Game, Patch } from "../../../kitor/src/store/store";
import type { GameSummary } from "../../ui/src/types";
import { CatalogSourceType, PatchLanguage, Platform, SourceType, Store, Tag } from "./index";

const tag: Tag = Tag.Detective;
const store: Store = Store.Steam;
const platform: Platform = Platform.MacOS;
const language: PatchLanguage = PatchLanguage.HebrewNiqqud;
const source: SourceType = SourceType.GitHub;
const catalogSource: CatalogSourceType = CatalogSourceType.Store;
// @ts-expect-error Misspelled tags must not compile.
const badTag: Tag = "clasisc";
// @ts-expect-error OS names must use registry wire values.
const badPlatform: Platform = "macos";
// @ts-expect-error Store entries cannot be used as downloadable patch sources.
const badSource: SourceType = CatalogSourceType.Store;
// @ts-expect-error UI presentation tags must use the registry vocabulary.
const badSummary: GameSummary["tags"] = ["clasisc"];
// @ts-expect-error App library metadata must reject unknown tags.
const badGameTags: Game["tags"] = ["clasisc"];
// @ts-expect-error App patch metadata must reject unknown sources.
const badPatchSource: Patch["source_type"] = "gitub";
// @ts-expect-error App patch metadata must reject unknown languages.
const badPatchLanguage: Patch["language"] = "hebrew";

void [
  tag,
  store,
  platform,
  language,
  source,
  catalogSource,
  badTag,
  badPlatform,
  badSource,
  badSummary,
  badGameTags,
  badPatchSource,
  badPatchLanguage,
];

// Launch choices use the same closed vocabulary as the generated native enums.
import type { NativeCommand } from "./schema";

// @ts-expect-error Per-title methods are not launch capabilities.
const badLaunchMethod: NativeCommand["method"] = "undertale";
// @ts-expect-error Arbitrary working-directory strings must not compile.
const badWorkingDirectory: NativeCommand["working_directory"] = "../other";
// @ts-expect-error Unknown custom preparation steps must not compile.
const badPreparation: NativeCommand["preparation"] = "unknown";
void [badLaunchMethod, badWorkingDirectory, badPreparation];

import type { DetectionMode } from "./index";
import type { DetectionFileCheck } from "./schema";

// @ts-expect-error Detection modes are reusable capabilities, not game names.
const badDetectionMode: DetectionMode = "disco_elysium";
const badDetectionCheck: DetectionFileCheck = {
  kind: "digest",
  path: "data",
  // @ts-expect-error Unsupported digest algorithms cannot be configured.
  algorithm: "sha1",
  values: [],
};
void [badDetectionMode, badDetectionCheck];
