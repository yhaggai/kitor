import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { PlatformSchema, validateRegistry } from "./schema";

const registry = JSON.parse(
  readFileSync(new URL("../../kitor/patches/registry.json", import.meta.url), "utf8")
);
const copy = () => structuredClone(registry);

describe("registry validation", () => {
  test("accepts the checked-in registry", () => {
    expect(validateRegistry(registry).games.length).toBeGreaterThan(0);
  });
  test.each(["macos", "win32", "android"])("rejects unsupported platform %s", (platform) => {
    expect(PlatformSchema.safeParse(platform).success).toBe(false);
  });
  test.each([
    ["unknown tag", (r: typeof registry) => r.games[0].tags.push("clasisc")],
    ["unknown source", (r: typeof registry) => (r.games[0].patches[0].source_type = "gitub")],
    ["unknown language", (r: typeof registry) => (r.games[0].patches[0].language = "hebrew")],
    [
      "unknown store",
      (r: typeof registry) => (r.games[0].patches[0].supported_builds[0].store = "steem"),
    ],
    ["unknown field", (r: typeof registry) => (r.games[0].platfroms = {})],
    [
      "unknown command platform",
      (r: typeof registry) =>
        (r.games[0].behavior.launch.original.commands = { macos: { program: "game" } }),
    ],
    ["invalid hash", (r: typeof registry) => (r.games[0].patches[0].checksum = "abc")],
    ["invalid date", (r: typeof registry) => (r.games[0].added_on = "2026-02-30")],
    ["duplicate identity", (r: typeof registry) => r.games.push(r.games[0])],
    [
      "legacy native adapters",
      (r: typeof registry) => (r.games[0].behavior.launch.original.native = "undertale"),
    ],
  ] as const)("rejects %s", (_name, mutate) => {
    const candidate = copy();
    mutate(candidate);
    expect(() => validateRegistry(candidate)).toThrow();
  });
  test("allows absent platform commands and the supported niqqud language", () => {
    const candidate = copy();
    candidate.games[0].patches[0].language = "he-x-nikud";
    candidate.games[0].behavior.launch.original = {
      commands: { windows: { program: "game.exe" } },
    };
    expect(() => validateRegistry(candidate)).not.toThrow();
  });
});
