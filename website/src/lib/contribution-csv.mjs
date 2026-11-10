export function parseCsv_(text) {
  const result = [];
  let start = 0,
    value = "",
    fields = [],
    quoted = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '"') {
      if (quoted && text[i + 1] === '"') {
        value += '"';
        i++;
      } else quoted = !quoted;
    } else if (c === "," && !quoted) {
      fields.push(value);
      value = "";
    } else if ((c === "\r" || c === "\n") && !quoted) {
      fields.push(value);
      value = "";
      const ending = c === "\r" && text[i + 1] === "\n" ? "\r\n" : c;
      if (ending.length === 2) i++;
      result.push({ fields: fields, raw: text.slice(start, i + 1), ending: ending });
      fields = [];
      start = i + 1;
    } else value += c;
  }
  if (quoted) throw new Error("Unterminated CSV quote.");
  if (start < text.length) {
    fields.push(value);
    result.push({ fields: fields, raw: text.slice(start), ending: "" });
  }
  return result;
}

export function encodeRow_(fields, ending) {
  return (
    fields.map((v) => (/[",\r\n]/.test(v) ? '"' + v.replace(/"/g, '""') + '"' : v)).join(",") +
    ending
  );
}

export function applyRows_(original, input) {
  const records = parseCsv_(original);
  if (input.length !== records.length)
    throw new Error(
      "Rows were added or removed. Restore the original row count before submitting."
    );
  const map = new Map();
  input.forEach((row, i) => {
    const normalized = row.map((x) => (x == null ? "" : String(x)));
    if (normalized.length !== 6) throw new Error("Expected six columns at sheet row " + (i + 1));
    const key = JSON.stringify(normalized.slice(0, 5));
    if (map.has(key)) throw new Error("Duplicate row identity at sheet row " + (i + 1));
    map.set(key, normalized);
  });
  const changes = [];
  const csv = records
    .map((record, i) => {
      const before = record.fields,
        next = map.get(JSON.stringify(before.slice(0, 5)));
      if (!next)
        throw new Error(
          "Source columns were edited or a row is missing (original record " +
            (i + 1) +
            "). Only edit Hebrew."
        );
      if (next[5] === before[5]) return record.raw;
      if (i < 3)
        throw new Error("Headers, instructions, and translation settings cannot be submitted.");
      if (!next[5].trim()) throw new Error("Empty Hebrew correction for " + before[4]);
      if (next[5].includes("\0")) throw new Error("A correction contains a NUL character.");
      const tokens = (s) =>
        (s.match(/\{[^{}]+\}|<\/?[A-Za-z][^>]*>|%\d*\$?[sdif]/g) || []).sort().join("\n");
      if (tokens(before[5]) !== tokens(next[5]))
        throw new Error("Formatting tokens changed for " + before[4]);
      changes.push({
        english: before[4],
        context: before[2] + " / " + before[3],
        before: before[5],
        after: next[5],
      });
      return encodeRow_(next, record.ending);
    })
    .join("");
  if (changes.length > 200)
    throw new Error("This POC accepts up to 200 changed lines per submission.");
  return { csv: csv, changes: changes };
}
