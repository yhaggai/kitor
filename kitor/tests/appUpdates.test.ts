import { describe, expect, test } from "bun:test";
import {
  type CheckResult,
  createAppUpdateStore,
  type UpdateTransport,
} from "../src/lib/appUpdates";

const available: CheckResult = {
  currentVersion: "0.1.0",
  canInstall: true,
  update: { version: "0.2.0", notes: "Better Hebrew support" },
};
function setup(overrides: Partial<UpdateTransport> = {}) {
  const calls = { checks: 0, installs: 0, restarts: 0 };
  const store = createAppUpdateStore({
    check: async () => {
      calls.checks++;
      return available;
    },
    install: async () => {
      calls.installs++;
    },
    restart: async () => {
      calls.restarts++;
    },
    ...overrides,
  });
  return { store, calls };
}

describe("app update flow", () => {
  test("StrictMode startup checks once and Later does not reprompt", async () => {
    const { store, calls } = setup();
    await Promise.all([store.getState().check(), store.getState().check()]);
    expect(calls.checks).toBe(1);
    expect(store.getState().open).toBe(true);
    store.getState().setOpen(false);
    await store.getState().check();
    expect(store.getState().open).toBe(false);
    await store.getState().check(true);
    expect(calls.checks).toBe(2);
    expect(store.getState().open).toBe(true);
  });
  test("offline startup remains quiet and a manual retry can recover", async () => {
    let offline = true;
    const { store } = setup({
      check: async () => {
        if (offline) throw new Error("Offline");
        return { ...available, update: null };
      },
    });
    await store.getState().check();
    expect(store.getState().status).toBe("error");
    expect(store.getState().open).toBe(false);
    offline = false;
    await store.getState().check(true);
    expect(store.getState().status).toBe("upToDate");
    expect(store.getState().error).toBeNull();
  });
  test("an update never installs without the user choosing it", async () => {
    const { store, calls } = setup();
    await store.getState().check();
    expect(calls.installs).toBe(0);
    await store.getState().install();
    expect(calls.installs).toBe(1);
    expect(calls.restarts).toBe(1);
  });
  test("progress supports unknown sizes and prevents duplicate install or dismissal", async () => {
    let finish!: () => void;
    const wait = new Promise<void>((resolve) => {
      finish = resolve;
    });
    const { store, calls } = setup({
      install: async (_version, progress) => {
        calls.installs++;
        progress({ event: "downloading", downloaded: 128, total: null });
        await wait;
        progress({ event: "verifying" });
        progress({ event: "installing" });
      },
    });
    await store.getState().check();
    const installing = store.getState().install();
    expect(store.getState().total).toBeNull();
    expect(store.getState().downloaded).toBe(128);
    store.getState().setOpen(false);
    expect(store.getState().open).toBe(true);
    await store.getState().install();
    await store.getState().check(true);
    expect(calls.installs).toBe(1);
    expect(calls.checks).toBe(1);
    finish();
    await installing;
    expect(calls.restarts).toBe(1);
  });
  test("failed verification never restarts and requires a new check", async () => {
    const { store, calls } = setup({
      install: async () => {
        throw new Error("Signature invalid");
      },
    });
    await store.getState().check();
    await store.getState().install();
    expect(store.getState().status).toBe("error");
    expect(store.getState().update).toBeNull();
    expect(calls.restarts).toBe(0);
    await store.getState().check(true);
    expect(store.getState().status).toBe("available");
  });
  test("restart failure preserves installed state without reinstalling", async () => {
    const { store, calls } = setup({
      restart: async () => {
        calls.restarts++;
        throw new Error("Restart failed");
      },
    });
    await store.getState().check();
    await store.getState().install();
    expect(store.getState().status).toBe("ready");
    await store.getState().check(true);
    await store.getState().install();
    await store.getState().restart();
    expect(calls.installs).toBe(1);
    expect(calls.restarts).toBe(2);
  });
  test("development builds can check but cannot install", async () => {
    const { store, calls } = setup({ check: async () => ({ ...available, canInstall: false }) });
    await store.getState().check();
    await store.getState().install();
    expect(calls.installs).toBe(0);
  });
});
