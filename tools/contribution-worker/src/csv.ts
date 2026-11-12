import { HttpError, hash } from "./shared";

type CsvRecord = { fields: string[]; start: number; end: number };
export function records(source: string): CsvRecord[] {
  const out: CsvRecord[] = [];
  let fields: string[] = [],
    field = "",
    start = 0,
    quoted = false,
    closed = false;
  for (let i = 0; i < source.length; i++) {
    const c = source[i];
    if (quoted) {
      if (c === '"') {
        if (source[i + 1] === '"') {
          field += '"';
          i++;
        } else {
          quoted = false;
          closed = true;
        }
      } else field += c;
      continue;
    }
    if (c === '"' && !field && !closed) {
      quoted = true;
      continue;
    }
    if (c === ",") {
      fields.push(field);
      field = "";
      closed = false;
      continue;
    }
    if (c === "\r" || c === "\n") {
      if (c === "\r" && source[i + 1] === "\n") i++;
      fields.push(field);
      out.push({ fields, start, end: i + 1 });
      fields = [];
      field = "";
      closed = false;
      start = i + 1;
      continue;
    }
    if (closed || c === '"') throw new HttpError(400, "Invalid CSV quoting");
    field += c;
  }
  if (quoted) throw new HttpError(400, "Unclosed CSV field");
  if (start < source.length) {
    fields.push(field);
    out.push({ fields, start, end: source.length });
  }
  if (
    out.some((r) => r.fields.length !== 6) ||
    out.length < 4 ||
    JSON.stringify(out[0].fields) !==
      JSON.stringify(["Character", "ID", "File", "Context", "English", "Hebrew"])
  )
    throw new HttpError(400, "Unsupported Drifter CSV");
  return out;
}
const tokens = (s: string) => JSON.stringify((s.match(/\{[^}]*\}|<[^>]*>|%[sdif]/g) || []).sort());
export async function apply(source: string, edits: { [key: string]: string }): Promise<string> {
  const rows = records(source);
  const seen = new Set<string>();
  const found = new Set<string>();
  let output = "";
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i];
    let raw = source.slice(row.start, row.end);
    if (i >= 3) {
      const key = await hash(JSON.stringify(row.fields.slice(0, 5)));
      if (seen.has(key)) throw new HttpError(400, "Duplicate translation identity");
      seen.add(key);
      if (Object.hasOwn(edits, key)) {
        found.add(key);
        const value = edits[key];
        if (tokens(value) !== tokens(row.fields[5]))
          throw new HttpError(400, "Keep existing formatting and control tokens");
        if (value !== row.fields[5]) {
          const fields = [...row.fields];
          fields[5] = value;
          const ending = raw.endsWith("\r\n") ? "\r\n" : raw.endsWith("\n") ? "\n" : "";
          raw =
            fields.map((f) => (/[,"\r\n]/.test(f) ? `"${f.replaceAll('"', '""')}"` : f)).join(",") +
            ending;
        }
      }
    }
    output += raw;
  }
  if (found.size !== Object.keys(edits).length) throw new HttpError(400, "Unknown translation key");
  return output;
}
export type Correction = {
  email_opt_in?: boolean;
  base_sha: string;
  digest: string;
  edits: { [key: string]: string };
};
export function validate(b: Record<string, unknown>): asserts b is Correction {
  if (
    !b ||
    Object.keys(b).some((key) => !["base_sha", "digest", "edits", "email_opt_in"].includes(key)) ||
    (b.email_opt_in !== undefined && typeof b.email_opt_in !== "boolean") ||
    typeof b.base_sha !== "string" ||
    !/^[a-f0-9]{40}$/.test(b.base_sha) ||
    typeof b.digest !== "string" ||
    !/^[a-f0-9]{64}$/.test(b.digest) ||
    !b.edits ||
    Array.isArray(b.edits) ||
    typeof b.edits !== "object"
  )
    throw new HttpError(400, "Invalid corrections");
  const entries = Object.entries(b.edits);
  if (!entries.length || entries.length > 200)
    throw new HttpError(400, "Submit between 1 and 200 corrections");
  for (const [key, value] of entries)
    if (
      !/^[a-f0-9]{64}$/.test(key) ||
      typeof value !== "string" ||
      !value.trim() ||
      new TextEncoder().encode(value).length > 16000 ||
      value.includes("\0")
    )
      throw new HttpError(400, "Invalid correction");
}
export function manifestChecksum(source: string, digest: string): string {
  const manifest = JSON.parse(source);
  const file = manifest.files.filter(
    (f: { source?: { path?: string; sha256?: string } }) =>
      f.source?.path === "payload/translation.csv"
  );
  if (file.length !== 1 || !/^[a-f0-9]{64}$/.test(file[0].source.sha256))
    throw new HttpError(400, "Invalid translation manifest");
  const old = file[0].source.sha256;
  const matches = [...source.matchAll(/"sha256"\s*:\s*"([a-f0-9]{64})"/g)].filter(
    (m) => m[1] === old
  );
  if (matches.length !== 1) throw new HttpError(400, "Ambiguous payload checksum");
  const m = matches[0],
    at = m.index! + m[0].lastIndexOf(old);
  return source.slice(0, at) + digest + source.slice(at + 64);
}
