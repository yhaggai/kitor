import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { DetectionMode } from "@kitor/registry";
import {
  DetectionFileCheckSchema,
  DetectionLayoutSchema,
  validateRegistry,
} from "@kitor/registry/schema";

const source = JSON.parse(
  readFileSync(new URL("../../kitor/patches/registry.json", import.meta.url), "utf8")
);
function registryWith(detection: unknown) {
  const value = structuredClone(source);
  value.games[0].behavior.detection = detection;
  return value;
}
describe("generic registry detection", () => {
  test("game-specific selectors have been replaced with generic evidence rules", () => {
    const games = validateRegistry(source).games;
    for (const id of [
      "disco-elysium",
      "owlboy",
      "tentacle",
      "day-of-the-tentacle-remastered",
      "monkey-se-native",
      "monkey2-se-native",
      "sq1",
      "sq2",
      "sq3",
      "sq4",
      "sq5",
      "sq6",
      "sq1sci",
      "torin",
      "toonstruck",
    ]) {
      const detection = games.find((game) => game.scummvm_id === id)?.behavior?.detection;
      expect(detection?.mode).toBe(DetectionMode.Layouts);
      expect(detection?.layouts?.length).toBeGreaterThan(0);
    }
    expect(() => validateRegistry(registryWith({ adapter: "disco_elysium" }))).toThrow();
  });
  test("Space Quest fingerprints retain the independent owned-test snapshot", () => {
    const fixtures = JSON.parse(
      readFileSync(
        new URL(
          "../../kitor/src-tauri/src/services/game_service/fixtures/space-quest.json",
          import.meta.url
        ),
        "utf8"
      )
    );
    const games = validateRegistry(source).games;
    for (const fixture of fixtures) {
      const rule = games.find((game) => game.scummvm_id === fixture.id)?.behavior?.detection
        ?.layouts?.[0];
      expect(rule?.required_files).toEqual(fixture.required);
      expect(rule?.file_checks).toEqual([
        { kind: "digest", path: fixture.signature, algorithm: "sha256", values: fixture.sha256 },
      ]);
    }
  });
  test.each([
    { root: "../game", required_files: ["marker"] },
    { root: ".", required_files: [] },
    { root: ".", required_files: ["marker"], ancestor: 4 },
    { root: ".", required_files: ["../outside"] },
    { root: ".", required_files: ["marker"], data: "../outside" },
  ])("rejects invalid layout %j", (layout) => {
    expect(DetectionLayoutSchema.safeParse(layout).success).toBe(false);
  });
  test.each([
    { kind: "game_specific", path: "marker" },
    { kind: "prefix", path: "data", bytes: [256] },
    { kind: "text", path: "data", equals: "Game", normalization: "guess" },
    { kind: "digest", path: "data", algorithm: "sha1", values: ["a".repeat(64)] },
    { kind: "digest", path: "data", algorithm: "sha256", values: ["a".repeat(64)], bytes: 0 },
  ])("rejects unknown or malformed evidence %j", (check) => {
    expect(DetectionFileCheckSchema.safeParse(check).success).toBe(false);
  });
  test("rejects empty strict detection and algorithm/hash mismatches", () => {
    expect(() => validateRegistry(registryWith({ mode: DetectionMode.Layouts }))).toThrow();
    expect(() =>
      validateRegistry(
        registryWith({
          mode: DetectionMode.Layouts,
          layouts: [
            {
              root: ".",
              required_files: ["data"],
              file_checks: [
                { kind: "digest", path: "data", algorithm: "md5", values: ["a".repeat(64)] },
              ],
            },
          ],
        })
      )
    ).toThrow();
  });
});
