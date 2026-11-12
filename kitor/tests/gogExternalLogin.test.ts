import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";

const script = readFileSync(
  new URL("../src-tauri/src/commands/gog_external_login.js", import.meta.url),
  "utf8"
);

function setup(responses: unknown[], origin = "https://login.gog.com") {
  let handler: (event: unknown) => Promise<void>;
  const requests: Array<{ path: string; options: RequestInit }> = [];
  const opened: string[] = [];
  const redirects: string[] = [];
  const status = { textContent: "", style: {}, setAttribute() {} };
  runInNewContext(script, {
    URL,
    AbortController,
    window: {
      location: {
        origin,
        replace: (url: string) => redirects.push(url),
        assign: (url: string) => opened.push(url),
      },
    },
    document: {
      addEventListener: (_: string, fn: typeof handler) => {
        handler = fn;
      },
      createElement: () => status,
      body: { prepend() {} },
    },
    setTimeout: (fn: () => void, delay: number) => {
      if (delay === 1000) queueMicrotask(fn);
      return 1;
    },
    clearTimeout() {},
    fetch: async (path: string, options: RequestInit) => {
      requests.push({ path, options });
      const data = responses.shift();
      return { ok: data !== undefined, json: async () => data };
    },
  });
  return {
    requests,
    opened,
    redirects,
    status,
    async click(url = "https://external-accounts.gog.com/login/providers/google/init") {
      let intercepted = false;
      await handler?.({
        target: { closest: () => ({ getAttribute: () => url }) },
        preventDefault: () => {
          intercepted = true;
        },
        stopImmediatePropagation() {},
      });
      return intercepted;
    },
  };
}
const processResponse = {
  process_id: "attempt-id",
  process_init_url:
    "https://external-accounts.gog.com/login/providers/google/init?redirect_uri=example",
};

test("external browser login polls GOG and completes in the original webview", async () => {
  const flow = setup([
    processResponse,
    { status: "in_progress" },
    { status: "finished", login_token: "a+b&c" },
  ]);
  expect(await flow.click()).toBe(true);
  expect(flow.opened).toEqual([processResponse.process_init_url]);
  expect(flow.requests.map((r) => r.path)).toEqual([
    "/login_process/google",
    "/login_process/attempt-id",
    "/login_process/attempt-id",
  ]);
  expect(flow.requests[0].options.method).toBe("POST");
  expect(flow.requests.every((r) => r.options.credentials === "same-origin")).toBe(true);
  expect(flow.redirects).toEqual(["/on_login_success_external?login_token=a%2Bb%26c"]);
});

test("unrelated pages and links cannot start browser sign-in", async () => {
  expect(await setup([]).click("https://evil.example/login/providers/google/init")).toBe(false);
  expect(await setup([], "https://accounts.google.com").click()).toBe(false);
});

test("invalid destinations are rejected and a failed attempt can be retried", async () => {
  const flow = setup([{ ...processResponse, process_init_url: "https://evil.example/" }]);
  await flow.click();
  expect(flow.opened).toHaveLength(0);
  expect(flow.status.textContent).toContain("try again");
  await flow.click();
  expect(flow.requests).toHaveLength(2);
  expect(flow.redirects).toHaveLength(0);
});

test("concurrent clicks do not create multiple login attempts", async () => {
  const flow = setup([processResponse, { status: "finished", login_token: "token" }]);
  await Promise.all([flow.click(), flow.click()]);
  expect(flow.opened).toHaveLength(1);
  expect(flow.requests).toHaveLength(2);
});
