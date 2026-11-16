import type { APIRoute } from "astro";
import { games } from "../lib/catalog";

export const GET: APIRoute = ({ site }) => {
  const pages = ["/", "/games/", ...games.map(({ id }) => `/games/${id}/`)];
  const urls = pages.flatMap((path) => [path, "/en" + path]);
  const entries = urls.map((path) => `  <url><loc>${new URL(path, site).href}</loc></url>`);

  return new Response(
    ['<?xml version="1.0" encoding="UTF-8"?>',
      '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
      ...entries,
      "</urlset>",
      "",
    ].join("\n"),
    { headers: { "Content-Type": "application/xml; charset=utf-8" } }
  );
};
