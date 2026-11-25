import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import {
  LaunchMethod,
  LaunchRoot,
  LaunchWorkingDirectory,
  NativePreparation,
} from "@kitor/registry";
import { CommandSchema, LaunchProfileSchema, validateRegistry } from "@kitor/registry/schema";

const registry = JSON.parse(
  readFileSync(new URL("../../kitor/patches/registry.json", import.meta.url), "utf8")
);
function withCommand(command: unknown) {
  const candidate = structuredClone(registry);
  candidate.games[0].behavior.launch.original = { commands: { osx: command } };
  return candidate;
}

describe("registry launch capabilities", () => {
  test("all native profiles use generic capabilities, with only custom settings preparation", () => {
    const games = validateRegistry(registry).games;
    for (const id of [
      "owlboy",
      "undertale",
      "drifter",
      "grim-fandango-remastered",
      "day-of-the-tentacle-remastered",
      "monkey-se-native",
      "monkey2-se-native",
    ]) {
      const profiles = games.find((game) => game.scummvm_id === id)?.behavior?.launch;
      expect(Object.keys(profiles?.original.commands ?? {}).length).toBeGreaterThan(0);
      expect(Object.keys(profiles?.patched.commands ?? {}).length).toBeGreaterThan(0);
    }
    const grim = games.find((game) => game.scummvm_id === "grim-fandango-remastered")?.behavior
      ?.launch;
    expect(grim?.original.commands?.windows?.preparation).toBeUndefined();
    expect(grim?.patched.commands?.windows?.preparation).toBe(
      NativePreparation.GrimHebrewPreferences
    );
    expect(LaunchProfileSchema.safeParse({ native: "owlboy" }).success).toBe(false);
  });
  test("bundle resolution, launch method, cwd and permission repair are configurable", () => {
    expect(() =>
      validateRegistry(
        withCommand({
          root: LaunchRoot.AppBundle,
          bundle_name: "Example.app",
          method: LaunchMethod.AppBundle,
          program: "Contents/MacOS/runner",
          working_directory: LaunchWorkingDirectory.BundleResources,
          executable_files: ["Contents/MacOS/runner"],
        })
      )
    ).not.toThrow();
  });
  test.each([
    { program: "../runner" },
    { program: "/bin/sh" },
    { program: "runner", root: "owlboy" },
    { program: "runner", method: "shell" },
    { program: "runner", working_directory: "../other" },
    { program: "runner", signatures: [{ path: "data", prefix: [256] }] },
    { program: "runner", signatures: [{ path: "data", prefix: [] }] },
    { program: "runner", executable_files: ["../other"] },
    { program: "runner", preparation: "unknown" },
  ])("rejects invalid structural capability %j", (command) => {
    expect(CommandSchema.safeParse(command).success).toBe(false);
  });
  test.each([
    { program: "runner", method: LaunchMethod.AppBundle },
    { program: "runner", root: LaunchRoot.AppBundle },
    { program: "runner", bundle_name: "Example.app" },
    { program: "runner", working_directory: LaunchWorkingDirectory.BundleParent },
    { program: "runner", root: LaunchRoot.AppBundle, bundle_name: "nested/Example.app" },
    {
      program: "runner",
      root: LaunchRoot.AppBundle,
      bundle_name: "Example.app",
      method: LaunchMethod.AppBundle,
      env: { FOO: "bar" },
    },
  ])("rejects incompatible capabilities %j", (command) => {
    expect(() => validateRegistry(withCommand(command))).toThrow();
  });
});

test("platform-specific capabilities and import requirements are checked at authoring time", () => {
  const candidate = structuredClone(registry);
  candidate.games[0].behavior.launch.original = { validate_on_import: true };
  expect(() => validateRegistry(candidate)).toThrow("Import validation requires native commands");
  expect(() =>
    validateRegistry(
      withCommand({ program: "runner", preparation: NativePreparation.GrimHebrewPreferences })
    )
  ).toThrow("requires Windows");
  candidate.games[0].behavior.launch.original = {
    commands: {
      windows: {
        root: LaunchRoot.AppBundle,
        bundle_name: "Game.app",
        program: "Contents/MacOS/Game",
      },
    },
  };
  expect(() => validateRegistry(candidate)).toThrow("require macOS");
});
