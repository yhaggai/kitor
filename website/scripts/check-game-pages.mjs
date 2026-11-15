import { readFile } from "node:fs/promises";

const website = new URL("../", import.meta.url);
const registryUrl = new URL("../kitor/patches/registry.json", website);
const dist = new URL("dist/", website);
const readJson = async (url) => JSON.parse(await readFile(url, "utf8"));

let ids;
try {
  const registry = await readJson(registryUrl);
  ids = registry.games.map((game) => game.scummvm_id);
} catch (error) {
  if (error.code !== "ENOENT") throw error;
  const catalog = await readJson(new URL("src/generated/catalog.json", website));
  ids = catalog.map((game) => game.id);
}

const errors = [];
const seen = new Set();
for (const id of ids) {
  if (typeof id !== "string" || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(id)) {
    errors.push(`Invalid game ID: ${id}`);
  } else if (seen.has(id)) {
    errors.push(`Duplicate game ID: ${id}`);
  } else {
    seen.add(id);
  }
}
if (ids.length === 0) errors.push("The game catalog is empty.");

const sitemap = await readFile(new URL("sitemap.xml", dist), "utf8");
const sitemapPaths = new Set(
  Array.from(sitemap.matchAll(/<loc>([^<]+)<\/loc>/g), ([, url]) => new URL(url).pathname)
);

for (const catalogPath of ["/games/", "/en/games/"]) {
  const gallery = await readFile(new URL(`${catalogPath.slice(1)}index.html`, dist), "utf8");
  const links = new Set(Array.from(gallery.matchAll(/\bhref="([^"]+)"/g), ([, href]) => href));

  for (const id of seen) {
    const pagePath = `${catalogPath}${id}/`;
    try {
      const page = await readFile(new URL(`${pagePath.slice(1)}index.html`, dist), "utf8");
      if (!page.includes('id="game-download"')) errors.push(`Missing game download on ${pagePath}`);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
      errors.push(`Missing static page: ${pagePath}`);
    }
    if (!links.has(pagePath)) errors.push(`Missing catalog link: ${pagePath}`);
    if (!sitemapPaths.has(pagePath)) errors.push(`Missing sitemap entry: ${pagePath}`);
  }
}

if (errors.length) {
  console.error(errors.join("\n"));
  process.exitCode = 1;
} else {
  console.log(`Verified Hebrew and English static pages, catalog links, and sitemap entries for ${ids.length} games.`);
}
