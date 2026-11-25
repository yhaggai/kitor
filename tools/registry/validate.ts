import { readFileSync, writeFileSync } from "node:fs";
import { registryJsonSchema } from "@kitor/registry/schema";
import { rustVocabulary } from "./rust";
import { validateRegistry } from "./schema";

const registryPath = new URL("../../kitor/patches/registry.json", import.meta.url);
const schemaPath = new URL("../../kitor/patches/registry.schema.json", import.meta.url);
const rustPath = new URL(
  "../../kitor/src-tauri/src/services/registry_vocabulary.rs",
  import.meta.url
);
const generatedRust = rustVocabulary();
const generated = `${JSON.stringify(registryJsonSchema(), null, 2)}\n`;
try {
  const registry = validateRegistry(JSON.parse(readFileSync(registryPath, "utf8")));
  if (process.argv.includes("--write-schema")) {
    writeFileSync(schemaPath, generated);
    writeFileSync(rustPath, generatedRust);
  } else if (
    readFileSync(schemaPath, "utf8") !== generated ||
    readFileSync(rustPath, "utf8") !== generatedRust
  ) {
    throw new Error("JSON Schema is stale. Run bun run registry:schema");
  }
  console.log(
    `Registry valid: ${registry.games.length} games, ${registry.games.reduce((n, game) => n + game.patches.length, 0)} patches`
  );
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
}
