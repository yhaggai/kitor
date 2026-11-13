# Kitor landing page

Hebrew RTL Astro website with React islands reusing the desktop GameBox component.

```sh
cd website
npm ci
npm run dev
npm run check
npm run build
```

The landing page is `/`; the complete supported-game gallery is `/games/`.

`npm run sync` runs before development and builds. It reads `../kitor/patches/registry.json`, copies the current desktop `GameBox.tsx` and its CSS without modification into `src/generated`, and uses the Cloudflare cover URLs from `../kitor/src/data/game-box-covers.json`. Edit the desktop sources, not generated files. The build fails if a supported game is missing Cloudflare artwork. The checked-in catalog snapshot allows standalone hosting builds; in the monorepo, the live app catalog is always authoritative.

The only desktop component addition is an optional `locale` prop for Hebrew viewer controls; English remains the default. The website provides the pure browser `cn` utility so the native Tauri API is not bundled.

Search uses English catalog titles and Hebrew search aliases in `src/lib/hebrew-titles.ts`. Platform filters reflect registry flags; they are not a claim of full playthrough validation. The landing page recommends a desktop installer using browser platform and architecture hints. It selects the highest published stable `app-vX.Y.Z` GitHub release that contains all three installers. The download section always shows all builds and falls back to verified installer links if the API is unavailable. Update the fallback version in `src/lib/downloads.ts` after publishing a new stable desktop release.

Generated night scenery is in `public/art`; source game art lives in the ignored local `assets/` workspace. Edge characters are lightweight SVG pixel illustrations, animated with IntersectionObserver and disabled motion for reduced-motion preferences.
