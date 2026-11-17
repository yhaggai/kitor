export const MAX_DESCRIPTION_LENGTH = 12_000;
export const MAX_DESCRIPTION_FEED_BYTES = 512 * 1024;

export function isDescriptionText(text: unknown): text is string {
  return typeof text === "string" && text.length <= MAX_DESCRIPTION_LENGTH && text.trim() !== "";
}

export function buildDescriptionFeed(catalog: unknown): string {
  if (!catalog || typeof catalog !== "object" || Array.isArray(catalog)) {
    throw new Error("Invalid description catalog");
  }
  const descriptions = Object.fromEntries(
    Object.entries(catalog).map(([id, entry]) => [id, entry?.description ?? null])
  );
  const json = JSON.stringify({ schemaVersion: 1, descriptions });
  parseDescriptionFeed(json);
  return json;
}

export function parseDescriptionFeed(json: string): Map<string, string> {
  if (new TextEncoder().encode(json).length > MAX_DESCRIPTION_FEED_BYTES) {
    throw new Error("Description feed is too large");
  }
  const feed = JSON.parse(json);
  if (
    feed?.schemaVersion !== 1 ||
    !feed.descriptions ||
    typeof feed.descriptions !== "object" ||
    Array.isArray(feed.descriptions)
  ) {
    throw new Error("Invalid description feed");
  }

  const descriptions = new Map<string, string>();
  for (const [id, text] of Object.entries(feed.descriptions)) {
    if (!/^[a-z0-9][a-z0-9_-]*$/.test(id) || !isDescriptionText(text)) {
      throw new Error(`Invalid description for ${id}`);
    }
    descriptions.set(id, text);
  }
  return descriptions;
}
