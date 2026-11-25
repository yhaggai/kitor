import { access, copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";

const root = fileURLToPath(new URL("..", import.meta.url));
const repo = path.resolve(root, "..");
const generated = path.join(root, "src/generated");
await mkdir(generated, { recursive: true });
try {
  await access(path.join(repo, "kitor/patches/registry.json"));
} catch {
  await access(path.join(generated, "catalog.json"));
  console.log("Using bundled catalog and shared component snapshot.");
  process.exit(0);
}
const read = async (p) => JSON.parse(await readFile(path.join(repo, p), "utf8"));
const registry = await read("kitor/patches/registry.json");
const artwork = await read("kitor/src/data/game-box-covers.json");
const descriptions = await read("kitor/src/data/game-descriptions.he.json");
const galleries = await read("kitor/src/data/game-galleries.json");
const catalog = [];
for (const game of registry.games) {
  const covers = artwork[game.scummvm_id] ?? { he: { src: "" }, en: { src: "" } };
  for (const lang of ["he", "en"]) {
    const src = covers[lang].src;
    if (!src) continue;
    if (game.scummvm_id === "sq6" && lang === "he" && src === "/cover-sq6-he.jpg") {
      await copyFile(
        path.join(repo, "kitor/public", src.slice(1)),
        path.join(root, "public", src.slice(1))
      );
      continue;
    }
    const localCover =
      src === `/cover-${game.scummvm_id}.jpg` ||
      src === `/cover-${game.scummvm_id}-${lang}.svg` ||
      (game.scummvm_id === "monkey-se" && src === "/monkey-classic-en.jpg") ||
      (game.scummvm_id === "monkey2-se" && src === "/monkey2-classic-en.jpg");
    if (localCover && /^\/[a-z0-9-]+\.(jpg|svg)$/.test(src)) {
      await copyFile(
        path.join(repo, "assets/game-boxes/desktop-sources", src.slice(1)),
        path.join(root, "public", src.slice(1))
      );
      continue;
    }
    const url = new URL(src);
    if (url.protocol !== "https:" || url.hostname !== "assets.kitor.click")
      throw new Error(`Expected Cloudflare artwork for ${game.scummvm_id} (${lang})`);
  }
  const translation = game.patches.find((patch) => patch.language === "he");
  const description = descriptions[game.scummvm_id];
  catalog.push({
    id: game.scummvm_id,
    added_on: game.added_on,
    title: game.title,
    title_he: game.title_he,
    covers,
    platforms: game.platforms,
    hebrew: Boolean(translation),
    descriptionHe: description?.description,
    descriptionEn: description?.descriptionEn,
    sourceUrl: description?.sourceUrl,
    screenshot: galleries[game.scummvm_id]?.[0]?.src,
    translationCredit: translation?.author,
    translationVersion: translation?.version,
    translationBuilds: translation?.supported_builds?.map(({ store, platform, build_id, manifest_id }) => ({
      store,
      platform,
      buildId: build_id,
      manifestId: manifest_id,
    })),
    steamAppId: game.steam_app_id,
    gogSlug: game.gog?.slug,
    gogProductId: game.gog?.id,
  });
}
await writeFile(path.join(generated, "catalog.json"), JSON.stringify(catalog, null, 2) + "\n");
// The desktop remains the maintained source; the website supplies its own translator.
const components = path.join(repo, "kitor/src/components");
const gameBox = await readFile(path.join(components, "GameBox.tsx"), "utf8");
await writeFile(
  path.join(generated, "GameBox.tsx"),
  gameBox.replace('from "@kitor/ui"', 'from "../lib/game-box-translation"')
);
await copyFile(path.join(components, "GameBox.module.css"), path.join(generated, "GameBox.module.css"));
await copyFile(
  path.join(repo, "packages/ui/src/assets/kitor-logo.svg"),
  path.join(generated, "kitor-logo.svg")
);
await sharp(path.join(root, "public/art/night.png"))
  .resize({ width: 1920, withoutEnlargement: true })
  .webp({ quality: 85 })
  .toFile(path.join(root, "public/art/night.webp"));
console.log(`Synced shared GameBox and ${catalog.length} supported games.`);
