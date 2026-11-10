const fs = require("node:fs"),
  vm = require("node:vm"),
  assert = require("node:assert/strict");
const context = {};
vm.createContext(context);
vm.runInContext(fs.readFileSync(__dirname + "/Code.gs", "utf8"), context);
const original = fs.readFileSync(
  __dirname + "/../../patches-workspace/drifter/he/payload/translation.csv",
  "utf8"
);
const records = context.parseCsv_(original);
const rows = records.map((r) => Array.from(r.fields));
assert.equal(records.length, 9802);
assert.equal(context.applyRows_(original, rows).csv, original);
const edited = rows.map((r) => r.slice());
edited[7][5] = "רדסב";
const result = context.applyRows_(original, edited);
assert.equal(result.changes.length, 1);
assert.equal(
  result.csv,
  original.replace(",1,Room- Title,ContinueGame,Ok,רושיא", ",1,Room- Title,ContinueGame,Ok,רדסב")
);
assert.equal(context.applyRows_(original, edited.slice().reverse()).csv, result.csv);
const multiline = edited.map((r) => r.slice());
multiline[7][5] = 'a,"quoted"\nsecond line';
assert.equal(
  context.parseCsv_(context.applyRows_(original, multiline).csv)[7].fields[5],
  multiline[7][5]
);
assert.throws(() => context.applyRows_(original, edited.slice(1)), /Rows were/);
const bad = edited.map((r) => r.slice());
bad[7][4] = "changed source";
assert.throws(() => context.applyRows_(original, bad), /Source columns/);
const duplicate = edited.map((r) => r.slice());
duplicate[8] = duplicate[7];
assert.throws(() => context.applyRows_(original, duplicate), /Duplicate/);
const setting = rows.map((r) => r.slice());
setting[2][5] = "N";
assert.throws(() => context.applyRows_(original, setting), /settings/);
const empty = rows.map((r) => r.slice());
empty[7][5] = "";
assert.throws(() => context.applyRows_(original, empty), /Empty Hebrew/);
console.log(
  "PASS: full 9,802-record roundtrip, one-line diff, sorting, multiline quoting, missing/duplicate/source/settings/empty rejection."
);
