# Kitor landing page

Hebrew RTL Astro website with React islands reusing the desktop GameBox component.

```sh
cd website
npm ci
npm run dev
npm run check
npm run build
```

The landing page is `/` and the catalog is `/games/`. Every game in `kitor/patches/registry.json` gets static pages at `/games/<id>/` and `/en/games/<id>/` on the next website build. Use a stable game ID and add the artwork and any available descriptions or screenshots to the shared desktop data. There is no separate website entry or page to create. A record added only to `patches-metadata/` is a catalog candidate until it is added to the app registry.

`npm run sync` runs before development and builds. It projects the app registry and shared artwork, descriptions, and screenshots into `src/generated/catalog.json` and copies the shared desktop GameBox component. Edit the desktop sources and regenerate the checked-in snapshot for standalone hosting builds.

`npm run build` checks that every registered game has both static pages, a download button, a link from each catalog, and both sitemap entries. The website validation workflow runs this build on pull requests that change the website or its shared inputs. The generated pages become live when the updated website is deployed.

The only desktop component addition is an optional `locale` prop for Hebrew viewer controls; English remains the default. The website provides the pure browser `cn` utility so the native Tauri API is not bundled.

Catalog search accepts English and Hebrew titles. The landing page recommends a desktop installer using browser platform and architecture hints. It selects the highest published stable `app-vX.Y.Z` GitHub release that contains all three installers. The download section always shows all builds and falls back to verified installer links if the API is unavailable. Update the fallback version in `src/lib/downloads.ts` after publishing a new stable desktop release.

Generated night scenery is in `public/art`; source game art lives in the ignored local `assets/` workspace. Edge characters are lightweight SVG pixel illustrations, animated with IntersectionObserver and disabled motion for reduced-motion preferences.

## Website analytics

Production builds use the Kitor GA4 web stream (`G-1EVLL5CNSE`). Set `PUBLIC_GA_MEASUREMENT_ID` to override the stream ID or set it to an empty value to disable analytics for a build. Development skips the Google tag.

The Hebrew and English landing pages send `installer_download` when a visitor clicks a direct installer link. The `installer_platform` parameter is `windows-x86_64`, `darwin-aarch64`, or `darwin-x86_64`. The `page_language` parameter is `he` or `en`.

In GA4 Admin, create event-scoped custom dimensions named **Installer platform** and **Page language**, using the corresponding parameter names above. In Explore, filter Event name to `installer_download` and break down Event count by Installer platform. Add Page language for the Hebrew and English split. For a deployment smoke test, click a direct installer link on each landing page and check the event and parameters in GA4 Realtime.
