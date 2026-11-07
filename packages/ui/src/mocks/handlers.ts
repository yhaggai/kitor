import { delay, HttpResponse, http } from "msw";
import { games } from "./games";
import { operationHandlers } from "./operations";

export const handlers = [
  ...operationHandlers,
  http.get("*/api/catalog/:collection", async ({ params }) => {
    await delay(150);
    return HttpResponse.json(
      params.collection === "discovery"
        ? games.map((game) =>
            game.id === "grim-fandango-remastered" ? { ...game, status: "unowned" } : game
          )
        : games.filter((game) => game.id !== "grim-fandango-remastered")
    );
  }),
];
export const emptyHandlers = [
  ...operationHandlers,
  http.get("*/api/catalog/:collection", () => HttpResponse.json([])),
];
export const errorHandlers = [
  ...operationHandlers,
  http.get("*/api/catalog/:collection", () =>
    HttpResponse.json({ message: "The catalog is temporarily unavailable." }, { status: 503 })
  ),
];
export const loadingHandlers = [
  ...operationHandlers,
  http.get("*/api/catalog/:collection", async () => {
    await delay("infinite");
    return HttpResponse.json([]);
  }),
];

export const retryHandlers = [
  http.get("*/api/catalog/:collection", () => new HttpResponse(null, { status: 503 }), {
    once: true,
  }),
  ...handlers,
];
