import { readFile, writeFile } from "node:fs/promises";
import { buildDescriptionFeed } from "../src/lib/descriptionFeedSchema.ts";

const source = new URL("../src/data/game-descriptions.he.json", import.meta.url);
const catalog = JSON.parse(await readFile(source, "utf8"));
const json = buildDescriptionFeed(catalog);
if (process.argv[2]) await writeFile(process.argv[2], json);
console.log(`Validated ${Object.keys(catalog).length} game descriptions`);
