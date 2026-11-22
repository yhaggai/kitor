export const catalogKey = "published-catalog";

export async function loadCatalog(kv, fallback) {
  const catalog = await kv.get(catalogKey, { type: "json" });
  if (catalog === null) return fallback;
  if (!Array.isArray(catalog.games) || !catalog.games.length || !Array.isArray(catalog.plans)) {
    throw new Error("Invalid published monitor catalog; retained previous snapshot");
  }
  return catalog;
}
