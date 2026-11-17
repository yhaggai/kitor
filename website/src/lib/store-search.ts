export interface SearchGame { gameID: string; external: string; steamAppID?: string }
export interface Store { storeID: string; storeName: string; isActive: number }
export interface Deal { storeID: string; dealID: string; price: string; retailPrice: string }
export interface GameOffers { info: { title: string; steamAppID?: string }; deals: Deal[] }
const base = "https://www.cheapshark.com/api/1.0/";
const cache = new Map<string, { expires: number; value: unknown }>();

async function request<T>(path: string, signal: AbortSignal): Promise<T> {
  const cached = cache.get(path);
  if (cached && cached.expires > Date.now()) return cached.value as T;
  const response = await fetch(base + path, { signal: AbortSignal.any([signal, AbortSignal.timeout(15000)]) });
  if (!response.ok) throw new Error(response.status === 429 ? "Too many requests. Please wait a minute and retry." : "Store search is temporarily unavailable. Please retry.");
  const value = await response.json();
  if (!value || value.error) throw new Error("The price provider could not complete this search. Please retry.");
  cache.set(path, { expires: Date.now() + 300000, value });
  return value as T;
}

export function findGames(game: { title: string; steamAppId?: number }, signal: AbortSignal) {
  const query = new URLSearchParams(game.steamAppId
    ? { steamAppID: String(game.steamAppId) }
    : { title: game.title, limit: "20" });
  return request<SearchGame[]>(`games?${query}`, signal);
}

export function loadOffers(id: string, signal: AbortSignal) {
  return Promise.all([
    request<GameOffers>(`games?id=${encodeURIComponent(id)}`, signal),
    request<Store[]>("stores", signal),
  ]);
}

export function groupOffers(stores: Store[], deals: Deal[]) {
  return stores.filter((store) => Number(store.isActive) === 1).map((store) => ({
    ...store,
    deals: deals.filter((deal) => deal.storeID === store.storeID && deal.price.trim() !== "" && Number.isFinite(Number(deal.price)) && Number(deal.price) >= 0)
      .sort((a, b) => Number(a.price) - Number(b.price)),
  })).sort((a, b) => {
    if (!a.deals.length || !b.deals.length) return Number(!a.deals.length) - Number(!b.deals.length) || a.storeName.localeCompare(b.storeName);
    return Number(a.deals[0].price) - Number(b.deals[0].price) || a.storeName.localeCompare(b.storeName);
  });
}

export function dealUrl(id: string) {
  // The API can return percent-encoded identifiers. Encode exactly once.
  let decoded = id;
  try { decoded = decodeURIComponent(id); } catch { /* Treat as a raw identifier. */ }
  return `https://www.cheapshark.com/redirect?dealID=${encodeURIComponent(decoded)}`;
}
