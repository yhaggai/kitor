import { expect, test } from "bun:test";
import { patchOptionLabel, selectedPatchOption } from "../src/lib/patchOptions";
import type { AvailablePatchInfo, InstalledPatchInfo } from "../src/store/store";

const available: AvailablePatchInfo[] = ["he", "he-x-nikud"].map((language) => ({
  language,
  version: "1",
  release_revision: 1,
  author: "",
  description: "",
}));
const installed: InstalledPatchInfo[] = [
  {
    id: 1,
    scummvm_id: "undertale",
    language: "he-x-nikud",
    version: "1",
    release_revision: 1,
    status: "active",
    installed_date: "2026-10-31",
  },
];

test("new installs default to unpointed Hebrew and offer both labels", () => {
  expect(selectedPatchOption(available, [], "")?.language).toBe("he");
  expect(available.map((p) => patchOptionLabel(p.language, "undertale"))).toEqual([
    "Hebrew without nikud",
    "Hebrew with nikud",
  ]);
});
test("installed nikud stays selected instead of the first catalog entry", () => {
  expect(selectedPatchOption(available, installed, "")?.language).toBe("he-x-nikud");
});
test("explicit choice survives restoring the installed option", () => {
  expect(selectedPatchOption(available, installed, "he")?.language).toBe("he");
  expect(selectedPatchOption(available, [], "he")?.language).toBe("he");
});
test("missing options and an empty catalog are handled", () => {
  expect(selectedPatchOption(available, [], "gone")?.language).toBe("he");
  expect(selectedPatchOption([], installed, "")).toBeUndefined();
});
