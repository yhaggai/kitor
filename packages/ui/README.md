# @kitor/ui

Kitor's shared React design system: warm charcoal surfaces, a cream sidebar with navy navigation,
and golden-orange actions, and artwork-led collections. The desktop app imports this package for its shell, collections, details, artwork,
and design tokens. Storybook supplies simulated data for isolated review.

## Run

From the repository root:

```sh
bun install
bun run storybook          # http://localhost:6006
bun run ui:check           # package TypeScript check
bun run storybook:build    # packages/ui/storybook-static
bun run --cwd kitor build  # desktop frontend with shared UI
```

## Consume

```tsx
import { AppShell, CollectionView, GameDetailsView, useGameNavigation } from "@kitor/ui";
import "@kitor/ui/styles.css";
```

The private, source-exported package can be compiled by a consumer's Vite build.
React and React DOM are peer dependencies. Public exports live in `src/index.ts`.
Native Tauri models, authentication, install/restore flows, and filesystem access
remain in `kitor`. `kitor/src/lib/presentation.ts` adapts native models to `GameSummary`; the app
connects action slots to its existing native commands and dialogs.

## Foundations

`src/styles.css` is the source of truth for `--kt-*` tokens. The desktop Tailwind theme maps to these tokens, including portaled dialogs.

| Role | Token | Value |
| --- | --- | --- |
| Canvas | `--kt-bg` | `#1d1c19` |
| Navigation | `--kt-sidebar` | `#feefce` (logo cream) |
| Raised surface | `--kt-surface` | `#27251f` |
| Text | `--kt-text` | `#feefce` (navy inside sidebar) |
| Supporting text | `--kt-muted` | `#b8b1a3` |
| Border | `--kt-border` | `#49443a` |
| Primary action / focus | `--kt-accent` | `#ffac4e` |
| Brand navy / active navigation | `--kt-brand-navy` | `#123b59` |
| Brand orange / selection marker | `--kt-brand-orange` | `#f47918` |
| Ready | `--kt-success` | `#afc990` |
| Needs attention | `--kt-danger` | `#f2a49b` |

Spacing follows 4, 8, 12, 16, 24, 32, 48px. Controls use 8px corners, surfaces
10px. The font stack is Inter, Noto Sans Hebrew, Arial, sans-serif; no external
font request is required. Hebrew content uses automatic direction and logical
alignment. Status always includes text, not only color. At narrow widths,
navigation moves above the content and game details stack vertically.

## Brand

`BrandLogo` uses the user-supplied Hebrew Kitor wordmark and steam illustration.
The original PNG is preserved in `src/assets/kitor-logo.png`. The component uses
`src/assets/kitor-logo.svg`, a 45-path vector trace with no embedded bitmap or font
dependency. Paper grain is simplified into a flat palette while preserving the
cream background, navy lettering and orange details. Regenerate it with
`uv run --with pillow --with vtracer python packages/ui/tools/vectorize_logo.py`. Do not crop the steam or recolor
the logo. It appears in the shared sidebar and its own Foundations story.

## Components and views

- `Button`: primary, secondary, ghost, disabled, standard button attributes.
- `AppShell`: navigation, skip link, current-page indication, footer slot.
- `GameCover`: original six-face 3D geometry, bilingual artwork, spine and lid, image-error fallback, shared animation target.
- `GameBox`: inline drag/keyboard rotation, Hebrew/English cover selection, bounded zoom and reset. Changing cover language does not change the game installation.
- `GameTile`: one keyboard-accessible entry into the dedicated game view.
- `StatusBadge`: installed, available, unowned, attention, original language.
- `CollectionView`: Library or Discovery, search, filters, loading/error/empty states.
- `GameDetailsView`: artwork, action slot, metadata, keyboard-operable information tabs.
- `useGameNavigation`: shared cover zoom, detail focus, Back focus/scroll restoration.

Keep a collection mounted inside a `hidden` wrapper while showing details to
preserve its filters and search. Place both under the hook's `root` ref. The
380ms cover transition uses the Web Animations API, so it does not depend on
View Transitions support in the native WebKit view. Reduced-motion preferences
skip the animation. Navigation is in-app state, not a URL/deep-link router.

## Storybook and MSW

Foundations documents tokens, typography, spacing and motion. Component stories
cover variants, missing artwork, Hebrew and long titles. View stories show Library,
Discovery, both detail flows, loading, empty and server error states.

`CatalogDemo` is a story-only HTTP adapter. It fetches `/api/catalog/:collection`;
MSW handlers in `src/mocks/handlers.ts` intercept it and return explicit fixtures.
No native commands, store accounts, or real installation actions run in Storybook.
Individual component stories use fixture props; full collection stories use MSW.
Mock state and credits are illustrative, not claims about a user's installation.

Mocking is configured only in `.storybook/preview.tsx`, following the
[Storybook MSW v2 integration](https://storybook.js.org/docs/9/writing-stories/mocking-data-and-modules/mocking-network-requests).
Standalone previews use the service worker. Embedded previews use the official
`@mswjs/interceptors` FetchInterceptor with MSW `getResponse` and the exact same
per-story handlers. This avoids WebKit iframe service-worker client routing issues.
No mock code is exported by the component package.

After updating MSW, regenerate the checked-in worker:

```sh
bunx --no-install msw init packages/ui/public --no-save
```

`ZoomAndBack` and `DiscoveryZoomAndBack` contain executable Storybook play tests
for navigation, detail focus, translation tabs, search preservation, and restored
card focus. `RetryAfterError` verifies recovery from an MSW 503 response. Open those stories to run them. The accessibility addon is enabled.
`RotateFlipAndZoom` checks the 3D box keyboard, language and zoom controls.
Native launch/install behavior still requires the desktop backend to validate.

Cover fixtures are local copies of existing Kitor artwork, used only for preview.
See `public/covers/sources.json` for provenance. They do not introduce new artwork
licenses or imply ownership of the underlying games.

## Desktop feature review

Start at **Flows / Desktop features / Full App**. Library and Discovery use the
same connected controller. Game management is in the gear dropdown beside Play;
Overview and Translation remain tabs. Reusable `ManageMenu`, `FlowDialog`,
`OperationProgress`, `ActionCard` and `SettingsView` are exported by `@kitor/ui`.

| Existing desktop surface | Review coverage |
| --- | --- |
| GameLibrary / AddGameDialog | Search, filters, automatic Steam/GOG scan, selected folder detection, no match, warnings, translation update check, remove all |
| GameCard | Play original/translated, edition selection, install/update with release notes, progress, retry, restore backup, refresh health, missing files, open folder, remove without deleting files |
| PatchBrowser / PurchaseGuide | Description and credits, Steam/GOG store actions, downloads, folder selection, ownership gate, unsupported edition, hidden progress, library handoff |
| SteamAuthCard | Connect/cancel, disconnect, API key, refresh, private-library error, privacy settings |
| GogAuthCard / GithubAuthCard | Sign-in/out, refresh library, configure/remove token |
| Settings / ScummVMDialog | Required runtime, install/update/uninstall, open runtime/folder |
| AppUpdateCard / AppUpdateDialog | Check, up-to-date, available/release notes, download/verify/install, restart |
| BugReportDialog | Game/details, screenshot selection/paste/removal and limits, optional diagnostics preview, copy / email preparation |

`mocks/operations.ts` intercepts operation requests using MSW. Four server responses
provide progress, then the story controller commits changes. Failure stories fail
the first attempt; retry succeeds without committing partial state. Download
progress can be hidden and reopened and continues across navigation. Story changes
reset review state. The fixtures, versions, ownership and translation editions
are illustrative, not real compatibility claims.

Native folder pickers, browser authentication, launches, store links, clipboard,
email preparation and restart are **simulated**; no native commands or external
account writes occur. Credentials, report text and file contents are not sent to
the operation endpoint. Desktop integration remains deferred until design review.

Interaction stories cover install/restore/remove through the gear menu,
failed-install retry, download hide/resume/library handoff, and keyboard navigation
of the gear dropdown. Existing zoom/back, catalog retry and 3D-box stories remain.

`GameBanner` adds a wide Steam-style hero above the command bar while retaining
`GameBox` in the details body. Jenny LeClue has the original Steam hero/logo and a
generated Hebrew banner, switchable in the preview. Other games currently use the
text fallback until their artwork is sourced and reviewed. Asset provenance and
the localization prompt are in `public/banners/sources.json`.

## Desktop banner integration

The app reads `kitor/src/data/game-banners.json`, keyed by the registry game ID.
Each entry contains `en`, optional `he`, and optional `logo` image URLs. Local
assets live in `kitor/public/banners/` and use `/banners/...` URLs. Missing or
failed banners display a title fallback. Add generated banners to this manifest
without changing the game action code.
