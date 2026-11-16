import { type Catalog, catalog } from "./catalogs";
import { HttpError, hash } from "./shared";

type CsvRecord = { fields: string[]; start: number; end: number };
// Slice whole fields rather than concatenating each character in large catalogs.
function* parse(source: string): Generator<CsvRecord> {
  const field = /"((?:[^"]|"")*)"|([^,"\r\n]*)/y;
  let at = source.startsWith("\uFEFF") ? 1 : 0;
  let start = 0;
  let fields: string[] = [];
  while (at < source.length) {
    field.lastIndex = at;
    const match = field.exec(source);
    if (!match) throw new HttpError(400, "Invalid CSV field");
    fields.push(match[1] === undefined ? match[2] : match[1].replaceAll('""', '"'));
    at = field.lastIndex;
    const c = source[at];
    if (c === ",") {
      at++;
      if (at === source.length) fields.push("");
      continue;
    }
    if (c === "\r" || c === "\n") {
      at += c === "\r" && source[at + 1] === "\n" ? 2 : 1;
    } else if (at < source.length) {
      throw new HttpError(400, "Invalid CSV quoting");
    }
    yield { fields, start, end: at };
    fields = [];
    start = at;
  }
  if (fields.length) yield { fields, start, end: at };
}

function* checked(source: string, config: Catalog) {
  let i = 0;
  for (const record of parse(source)) {
    if (
      record.fields.length !== config.header.length ||
      (i === 0 && record.fields.some((value, col) => value !== config.header[col]))
    )
      throw new HttpError(400, "Unsupported translation CSV");
    yield { ...record, i: i++ };
  }
  if (i <= config.skip) throw new HttpError(400, "Empty translation CSV");
}

export function records(source: string): CsvRecord[] {
  return Array.from(checked(source, catalog()), ({ fields, start, end }) => ({
    fields,
    start,
    end,
  }));
}

const marks = /[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]/g;
const unpoint = (text: string) => text.normalize("NFD").replace(marks, "");

export async function apply(
  source: string,
  edits: { [key: string]: string },
  config = catalog()
): Promise<string> {
  const drifter = config.id === "drifter" || config.id === "drifter-logical";
  const pattern = new RegExp(config.tokens, "g");
  const tokens = (s: string) => JSON.stringify((s.match(pattern) || []).sort());
  const locked = new Set(config.locked_keys);
  const seen = new Set<string>();
  const found = new Set<string>();
  const parts: string[] = [];
  let end = 0;
  for (const row of checked(source, config)) {
    if (row.i < config.skip) continue;
    const id = drifter
      ? await hash(JSON.stringify(row.fields.slice(0, 5)))
      : row.fields[config.key];
    if (!id || seen.has(id)) throw new HttpError(400, "Duplicate or missing translation identity");
    seen.add(id);
    const fields = row.fields;
    let changed = false;
    for (const col of config.editable) {
      const key = drifter ? id : `${col}:${id}`;
      if (!Object.hasOwn(edits, key)) continue;
      found.add(key);
      const value = edits[key];
      const before = fields[col];
      if (value === before) continue;
      if (locked.has(id) || (config.status !== undefined && fields[config.status] === "unchanged"))
        throw new HttpError(400, "This line is not editable in the current patch");
      if (
        (config.id !== "drifter" && /[\u202a-\u202e\u2066-\u2069]/.test(value)) ||
        (config.id.startsWith("undertale") && /[\r\n]/.test(value)) ||
        (config.id === "owlboy" &&
          before.match(/\|[\s\S]*$/)?.[0] !== value.match(/\|[\s\S]*$/)?.[0]) ||
        tokens(value) !==
          tokens(before || (config.english === undefined ? "" : fields[config.english]))
      )
        throw new HttpError(400, "Keep existing formatting and control tokens");
      fields[col] = value;
      changed = true;
    }
    if (!changed) continue;
    if (config.editable.length === 2) {
      const [regular, pointed] = config.editable.map((col) => fields[col]);
      if (
        (config.id === "owlboy"
          ? regular !== pointed.replace(marks, "")
          : unpoint(regular) !== unpoint(pointed)) ||
        /[\u0591-\u05bd\u05bf\u05c1-\u05c2\u05c4-\u05c5\u05c7]/.test(regular)
      )
        throw new HttpError(
          400,
          "Update both Hebrew versions so their wording matches without niqqud"
        );
    }
    const raw = source.slice(row.start, row.end);
    const ending = raw.endsWith("\r\n")
      ? "\r\n"
      : raw.endsWith("\n")
        ? "\n"
        : raw.endsWith("\r")
          ? "\r"
          : "";
    parts.push(
      source.slice(end, row.start),
      fields.map((f) => (/[,"\r\n]/.test(f) ? `"${f.replaceAll('"', '""')}"` : f)).join(","),
      ending
    );
    end = row.end;
  }
  if (found.size !== Object.keys(edits).length) throw new HttpError(400, "Unknown translation key");
  return end === 0 ? source : parts.join("") + source.slice(end);
}
export type Correction = {
  email_opt_in?: boolean;
  catalog_id?: string;
  base_sha: string;
  digest: string;
  edits: { [key: string]: string };
};
export function validate(b: Record<string, unknown>): asserts b is Correction {
  if (
    !b ||
    Object.keys(b).some(
      (key) => !["base_sha", "digest", "edits", "email_opt_in", "catalog_id"].includes(key)
    ) ||
    (b.catalog_id !== undefined && typeof b.catalog_id !== "string") ||
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
  const config = catalog(b.catalog_id);
  const entries = Object.entries(b.edits);
  if (!entries.length || entries.length > 200)
    throw new HttpError(400, "Submit between 1 and 200 corrections");
  for (const [key, value] of entries)
    if (
      (config.id === "drifter" || config.id === "drifter-logical"
        ? !/^[a-f0-9]{64}$/.test(key)
        : !/^[0-9]:[\s\S]{1,2048}$/.test(key)) ||
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
