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
await mkdir(path.join(root, "public/covers"), { recursive: true });
for (const game of registry.games) {
  const covers = artwork[game.scummvm_id];
  if (!covers) throw new Error(`Missing artwork for ${game.scummvm_id}`);
  const pair = {};
  for (const lang of ["he", "en"]) {
    const url = covers[lang].src;
    const suffix = url.includes("/covers/") ? url.slice(url.indexOf("/covers/") + 1) : null;
    const source = url.startsWith("/")
      ? path.join(repo, "kitor/public", url)
      : path.join(repo, "assets/game-boxes", suffix ?? "MISSING");
    const filename = `${game.scummvm_id}-${lang}.webp`;
    const output = path.join(root, "public/covers", filename);
    try {
      await access(source);
    } catch {
      // Fresh checkouts use the committed, optimized artwork snapshot.
      await access(output);
      pair[lang] = { ...covers[lang], src: `/covers/${filename}` };
      continue;
    }
    await sharp(source)
      .resize({ width: 600, withoutEnlargement: true })
      .webp({ quality: 88 })
      .toFile(path.join(root, "public/covers", filename));
    pair[lang] = { ...covers[lang], src: `/covers/${filename}` };
  }
  catalog.push({
    id: game.scummvm_id,
    title: game.title,
    covers: pair,
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
  `Synced shared GameBox and ${catalog.length} supported games (${catalog.length * 2} original covers).`
);
