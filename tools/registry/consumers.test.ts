import { describe, expect, test } from "bun:test";
import { CatalogSourceType, PatchLanguage, Platform, SourceType, Tag } from "@kitor/registry";
import {
  LanguageSchema,
  PlatformSchema,
  SourceTypeSchema,
  TagSchema,
} from "@kitor/registry/schema";
import { patchOptionLabel, selectedPatchOption } from "../../kitor/src/lib/patchOptions";
import {
  categoryLabel,
  categoryLabels,
  hasCategory,
  orderedCategories,
} from "../../packages/ui/src/categories";

describe("shared registry consumers", () => {
  test("all vocabulary constants are accepted by their schemas", () => {
    for (const value of Object.values(Tag)) expect(TagSchema.parse(value)).toBe(value);
    for (const value of Object.values(Platform)) expect(PlatformSchema.parse(value)).toBe(value);
    for (const value of Object.values(PatchLanguage))
      expect(LanguageSchema.parse(value)).toBe(value);
    for (const value of Object.values(SourceType))
      expect(SourceTypeSchema.parse(value)).toBe(value);
    expect(SourceTypeSchema.safeParse(CatalogSourceType.Store).success).toBe(false);
  });
  test("every tag has a category label and the established display order is preserved", () => {
    expect(Object.keys(categoryLabels).sort()).toEqual(Object.values(Tag).sort());
    expect(orderedCategories([{ tags: [Tag.Detective, Tag.HotKitor, Tag.Classic] }])).toEqual([
      Tag.HotKitor,
      Tag.Classic,
      Tag.Detective,
    ]);
    expect(categoryLabel(Tag.HotKitor)).toBe("Hot Kitor");
    expect(hasCategory([Tag.Classic], Tag.Detective)).toBe(false);
    expect(hasCategory(undefined, "all")).toBe(true);
  });
  test("niqqud choice selects and labels the matching translation", () => {
    const available = Object.values(PatchLanguage).map((language) => ({
      language,
      release_revision: 1,
      version: "1",
      author: "test",
      description: "",
    }));
    expect(selectedPatchOption(available, [], PatchLanguage.HebrewNiqqud)?.language).toBe(
      PatchLanguage.HebrewNiqqud
    );
    expect(patchOptionLabel(PatchLanguage.HebrewNiqqud)).toBe("Hebrew with nikud");
    expect(patchOptionLabel(PatchLanguage.Hebrew, "undertale")).toBe("Hebrew without nikud");
    expect(patchOptionLabel("")).toBe("");
  });
});
