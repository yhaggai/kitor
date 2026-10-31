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
const catalog = [];
for (const game of registry.games) {
  const covers = artwork[game.scummvm_id];
  if (!covers) throw new Error(`Missing artwork for ${game.scummvm_id}`);
  for (const lang of ["he", "en"]) {
    const src = covers[lang].src;
    if (src === `/cover-${game.scummvm_id}-${lang}.svg` && /^\/cover-[a-z0-9-]+\.svg$/.test(src)) {
      await copyFile(path.join(repo, "kitor/public", src.slice(1)), path.join(root, "public", src.slice(1)));
      continue;
    }
    const url = new URL(src);
    if (url.protocol !== "https:" || url.hostname !== "assets.kitor.click")
      throw new Error(`Expected Cloudflare artwork for ${game.scummvm_id} (${lang})`);
  }
  catalog.push({
    id: game.scummvm_id,
    title: game.title,
    covers,
    platforms: game.platforms,
    hebrew: game.patches.some((p) => p.language === "he"),
  });
}
await writeFile(path.join(generated, "catalog.json"), JSON.stringify(catalog, null, 2) + "\n");
// Mechanical snapshot: the desktop component remains the single maintained source.
for (const file of ["GameBox.tsx", "GameBox.module.css"])
  await copyFile(path.join(repo, "kitor/src/components", file), path.join(generated, file));
await sharp(path.join(root, "public/art/night.png"))
  .resize({ width: 1920, withoutEnlargement: true })
  .webp({ quality: 85 })
  .toFile(path.join(root, "public/art/night.webp"));
console.log(
  `Synced shared GameBox and ${catalog.length} supported games (${catalog.length * 2} Cloudflare covers).`
);
