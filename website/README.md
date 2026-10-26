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

`npm run sync` runs before development and builds. It reads `../kitor/patches/registry.json`, copies the current desktop `GameBox.tsx` and its CSS without modification into `src/generated`, and produces optimized WebP copies of the real English/Hebrew cover assets. Edit the desktop source, not generated files. When ignored local originals are unavailable, it uses the checked-in optimized covers. The build fails if a supported game has neither a local original nor an optimized cover. The checked-in snapshot allows standalone hosting builds; in the monorepo, the live app catalog is always authoritative.

The only desktop component addition is an optional `locale` prop for Hebrew viewer controls; English remains the default. The website provides the pure browser `cn` utility so the native Tauri API is not bundled.

Search uses English catalog titles and Hebrew search aliases in `src/lib/hebrew-titles.ts`. Platform filters reflect registry flags; they are not a claim of full playthrough validation. Download buttons lead to the release section and then the actual GitHub releases page, because installer availability must come from published releases.

Generated night scenery is in `public/art`; source game art lives in the ignored local `assets/` workspace. Edge characters are lightweight SVG pixel illustrations, animated with IntersectionObserver and disabled motion for reduced-motion preferences.
