import { afterEach, expect, mock, test } from "bun:test";
const invoke = mock((_command: string): Promise<unknown> => Promise.resolve(false));
mock.module("@tauri-apps/api/core", () => ({ invoke }));
const { useGogLibrary } = await import("../src/store/gog");
const { useAppStore } = await import("../src/store/store");
afterEach(() => { useGogLibrary.getState().clear(); invoke.mockReset(); });

test("startup loads saved ownership and shares concurrent requests", async () => {
  invoke.mockImplementation(async (command) => command === "check_gog_auth_status" ? true : [{ id: 7, title: "Owned", cover_url: null }]);
  const first = useGogLibrary.getState().refresh();
  expect(useGogLibrary.getState().refresh()).toBe(first);
  await first;
  expect(invoke).toHaveBeenCalledTimes(2);
  expect(useAppStore.getState().gogGames[0].id).toBe(7);
  expect(useGogLibrary.getState().loading).toBe(false);
});

test("logout discards outstanding ownership responses", async () => {
  let finish!: (value: unknown) => void;
  invoke.mockImplementation((command) => command === "check_gog_auth_status" ? Promise.resolve(true) : new Promise((resolve) => { finish = resolve; }));
  const request = useGogLibrary.getState().refresh();
  await Promise.resolve();
  useGogLibrary.getState().clear();
  finish([{ id: 7, title: "Old account" }]);
  await request;
  expect(useAppStore.getState().gogAuth.isAuthenticated).toBe(false);
  expect(useAppStore.getState().gogGames).toEqual([]);
});

test("failed library requests surface an error and can be retried", async () => {
  invoke.mockImplementation(async (command) => {
    if (command === "check_gog_auth_status") return true;
    throw new Error("Network unavailable");
  });
  await useGogLibrary.getState().refresh();
  expect(useGogLibrary.getState().error).toContain("Network unavailable");
  invoke.mockImplementation(async (command) => command === "check_gog_auth_status" ? true : []);
  await useGogLibrary.getState().refresh();
  expect(useGogLibrary.getState().error).toBeNull();
  expect(useAppStore.getState().gogGames).toEqual([]);
});
