/// <reference types="vite/client" />
import { FetchInterceptor } from "@mswjs/interceptors/fetch";
import type { Preview } from "@storybook/react-vite";
import { getResponse, type RequestHandler } from "msw";
import { initialize, mswLoader } from "msw-storybook-addon";
import "../src/styles.css";

// Embedded WebKit previews may associate fetches with the manager's client ID
// instead of the preview iframe, bypassing the service worker. MSW's official
// fetch interceptor resolves the same handlers inside that iframe.
const embedded = window.self !== window.top;
let activeHandlers: RequestHandler[] = [];
if (embedded) {
  const interceptor = new FetchInterceptor();
  interceptor.on("request", async ({ request, controller }) => {
    const response = await getResponse(activeHandlers, request);
    if (response) controller.respondWith(response);
  });
  interceptor.apply();
  import.meta.hot?.dispose(() => interceptor.dispose());
} else {
  initialize({ onUnhandledRequest: "bypass" });
}
const preview: Preview = {
  loaders: [
    async (context) => {
      if (!embedded) return mswLoader(context);
      activeHandlers = context.parameters.msw?.handlers ?? [];
      for (const handler of activeHandlers) handler.isUsed = false;
      return {};
    },
  ],
  decorators: [
    (Story, context) => (
      <div className="kitor-theme" key={context.id}>
        <Story />
      </div>
    ),
  ],
  parameters: {
    layout: "fullscreen",
    backgrounds: { options: { charcoal: { name: "Charcoal", value: "#1d1c19" } } },
    a11y: { test: "error" },
  },
  initialGlobals: { backgrounds: { value: "charcoal" } },
};
export default preview;
