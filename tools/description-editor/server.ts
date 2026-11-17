import { randomUUID } from "node:crypto";
import { fileURLToPath } from "node:url";
import {
  buildDescriptionFeed,
  isDescriptionText,
  MAX_DESCRIPTION_LENGTH,
} from "../../kitor/src/lib/descriptionFeedSchema.ts";
import registry from "../../kitor/patches/registry.json";

const repo = "yhaggai/kitor";
const base = "v1";
const catalogPath = "kitor/src/data/game-descriptions.he.json";
const origin = "http://127.0.0.1:4732";
const gamesById = new Map(registry.games.map((game) => [game.scummvm_id, game]));
const files = new Map([
  ["/", "index.html"],
  ["/editor.js", "editor.js"],
  ["/editor.css", "editor.css"],
]);

type Catalog = Record<string, { description: string }>;

async function github<T>(endpoint: string, method = "GET", body?: unknown): Promise<T> {
  const args = [
    "gh",
    "api",
    "--hostname",
    "github.com",
    "--method",
    method,
    `repos/${repo}/${endpoint}`,
  ];
  if (body) args.push("--input", "-");
  const process = Bun.spawn(args, {
    stdin: body ? new Response(JSON.stringify(body)) : "ignore",
    stdout: "pipe",
    stderr: "pipe",
    signal: AbortSignal.timeout(20_000),
  });
  const [stdout, stderr, code] = await Promise.all([
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
    process.exited,
  ]);
  if (code !== 0) throw new Error(stderr.trim() || "GitHub CLI request failed");
  return JSON.parse(stdout);
}

async function readCatalog(ref: string) {
  const file = await github<{ sha: string; content: string }>(`contents/${catalogPath}?ref=${ref}`);
  const text = Buffer.from(file.content, "base64").toString("utf8");
  return { sha: file.sha, catalog: JSON.parse(text) as Catalog };
}

async function submit(request: Request) {
  const input = await request.json();
  const { id, description, sha } = input ?? {};
  if (
    typeof id !== "string" ||
    !/^[a-z0-9][a-z0-9_-]*$/.test(id) ||
    typeof sha !== "string" ||
    !isDescriptionText(description)
  ) {
    return Response.json(
      { error: "Select a game and enter a valid description." },
      { status: 400 }
    );
  }

  const ref = await github<{ object: { sha: string } }>(`git/ref/heads/${base}`);
  const current = await readCatalog(ref.object.sha);
  if (current.sha !== sha) {
    return Response.json(
      { error: "The catalog changed on GitHub. Copy your draft, then reload before submitting." },
      { status: 409 }
    );
  }
  if (!Object.hasOwn(current.catalog, id) || current.catalog[id].description === description) {
    return Response.json(
      { error: "Select an existing game and change its description." },
      { status: 400 }
    );
  }

  current.catalog[id].description = description;
  buildDescriptionFeed(current.catalog);
  const content = Buffer.from(JSON.stringify(current.catalog, null, 2) + "\n").toString("base64");
  const branch = `codex/description-${id}-${randomUUID().slice(0, 8)}`;
  const title = `Update game description for ${id}`;
  await github("git/refs", "POST", { ref: `refs/heads/${branch}`, sha: ref.object.sha });
  try {
    await github(`contents/${catalogPath}`, "PUT", { message: title, content, sha, branch });
    const pr = await github<{ html_url: string }>("pulls", "POST", {
      title,
      head: branch,
      base,
      body: `Updates the Hebrew catalog description for \`${id}\`.`,
    });
    return Response.json({ url: pr.html_url });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`GitHub created branch ${branch}. ${message}`);
  }
}

export async function handleRequest(request: Request) {
  if (request.headers.get("host") !== new URL(origin).host) {
    return new Response("Invalid host", { status: 403 });
  }
  const { pathname } = new URL(request.url);
  try {
    if (request.method === "GET") {
      const name = files.get(pathname);
      if (name) {
        const file = Bun.file(fileURLToPath(new URL(name, import.meta.url)));
        return new Response(file, {
          headers: {
            "Content-Security-Policy":
              "default-src 'self'; object-src 'none'; frame-ancestors 'none'",
          },
        });
      }
      if (pathname === "/api/catalog") {
        const { catalog, sha } = await readCatalog(base);
        const games = Object.entries(catalog).map(([id, entry]) => {
          const game = gamesById.get(id);
          return {
            id,
            title: game?.title_he || game?.title || id,
            englishTitle: game?.title ?? "",
            description: entry.description,
          };
        });
        return Response.json(
          { games, sha, maxLength: MAX_DESCRIPTION_LENGTH },
          {
            headers: { "Cache-Control": "no-store" },
          }
        );
      }
    }
    if (request.method === "POST" && pathname === "/api/submit") {
      if (
        request.headers.get("origin") !== origin ||
        request.headers.get("content-type") !== "application/json"
      ) {
        return new Response("Invalid request origin or content type", { status: 403 });
      }
      return await submit(request);
    }
    return new Response("Not found", { status: 404 });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return Response.json({ error: message }, { status: 502 });
  }
}

if (import.meta.main) {
  Bun.serve({
    hostname: "127.0.0.1",
    port: 4732,
    maxRequestBodySize: 80 * 1024,
    fetch: handleRequest,
  });
  console.log(`Kitor description editor running at ${origin}`);
}
