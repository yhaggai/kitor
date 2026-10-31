import { afterEach, beforeEach, expect, mock, test } from "bun:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

const invoke = mock((_command: string, _args?: unknown): Promise<unknown> => Promise.resolve([]));
let report:
  | ((event: { payload: { game_id: string; step: string; percentage: number | null } }) => void)
  | undefined;
const unlisten = mock(() => {});
mock.module("@tauri-apps/api/core", () => ({ invoke, convertFileSrc: (path: string) => path }));
mock.module("@tauri-apps/api/event", () => ({
  listen: async (_event: string, callback: typeof report) => {
    report = callback;
    return unlisten;
  },
}));
const ask = mock(async () => false);
mock.module("@tauri-apps/plugin-dialog", () => ({ open: async () => null, ask }));
const { useGogDownload } = await import("../src/store/gogDownload");
const { useAppStore } = await import("../src/store/store");
const { GameInstallFolder } = await import("../src/components/dialogs/GameInstallFolder");
beforeEach(() => {
  useAppStore.setState({
    gogAuth: { isAuthenticated: true },
    gogGames: [
      { id: 1207658695, title: "Beneath a Steel Sky" },
      { id: 1, title: "Test game" },
    ],
  });
});
afterEach(() => {
  useGogDownload.setState({ busy: false, gameId: null, error: null, complete: false });
  useAppStore.setState({ games: [], highlightedGameId: null, activeTab: "patches" });
  invoke.mockReset();
  unlisten.mockClear();
  report = undefined;
});

test("folder preview shows the exact game subdirectories under the selected destination", () => {
  const html = renderToStaticMarkup(
    createElement(GameInstallFolder, {
      value: "/Volumes/Games/",
      onChange: () => {},
      folders: ["sq1-steam", "sq2-steam"],
    })
  );
  expect(html).toContain("/Volumes/Games/sq1-steam");
  expect(html).toContain("/Volumes/Games/sq2-steam");
  expect(html).toContain("Browse");
});

test("GOG install passes the chosen folder, reports progress and adds the unpatched game", async () => {
  const game = { id: 17, scummvm_id: "sky", installed_patches: [], is_patched: false };
  let finish!: (value: unknown) => void;
  invoke.mockImplementation((command) =>
    command === "install_gog_game"
      ? new Promise((resolve) => {
          finish = resolve;
        })
      : Promise.resolve([game])
  );
  const request = useGogDownload.getState().start(1207658695, "sky", "/Volumes/Games");
  await Promise.resolve();
  expect(invoke).toHaveBeenCalledWith("install_gog_game", {
    gameId: 1207658695,
    scummvmId: "sky",
    installRoot: "/Volumes/Games",
  });
  report?.({ payload: { game_id: "other", step: "wrong game", percentage: 90 } });
  expect(useGogDownload.getState().percentage).toBeNull();
  report?.({ payload: { game_id: "sky", step: "Downloading game…", percentage: 42 } });
  expect(useGogDownload.getState().percentage).toBe(42);
  await useGogDownload.getState().start(1, "other", "/other");
  expect(invoke).toHaveBeenCalledTimes(1);
  finish(game);
  await request;
  expect(useAppStore.getState().games).toEqual([game]);
  expect(useAppStore.getState().highlightedGameId).toBe(17);
  expect(useGogDownload.getState().complete).toBe(true);
  expect(useGogDownload.getState().busy).toBe(false);
  expect(unlisten).toHaveBeenCalledTimes(1);
  expect(invoke.mock.calls.map(([command]) => command)).toEqual([
    "install_gog_game",
    "get_games_with_patches",
  ]);
});

test("download failure leaves the library unchanged and allows retry", async () => {
  invoke.mockRejectedValue(new Error("Interrupted download"));
  await useGogDownload.getState().start(1, "sky", "/Games");
  expect(useAppStore.getState().games).toEqual([]);
  expect(useGogDownload.getState().error).toContain("Interrupted download");
  expect(useGogDownload.getState().complete).toBe(false);
  expect(useGogDownload.getState().busy).toBe(false);
  expect(unlisten).toHaveBeenCalledTimes(1);
});

// Render dialog contents without Radix's browser-only portal for the state checks.
const section = ({ children }: { children?: import("react").ReactNode }) =>
  createElement("section", null, children);
mock.module("../src/components/ui/dialog", () => ({
  Dialog: ({ open, children }: { open: boolean; children: import("react").ReactNode }) =>
    open ? createElement("div", { role: "dialog" }, children) : null,
  DialogContent: section,
  DialogDescription: section,
  DialogFooter: section,
  DialogHeader: section,
  DialogTitle: section,
}));
mock.module("../src/store/gogDownload", () => ({
  useGogDownload: Object.assign(() => useGogDownload.getState(), useGogDownload),
}));
const { GogDownloadDialog } = await import("../src/components/dialogs/GogDownloadDialog");
function renderDialog() {
  return renderToStaticMarkup(
    createElement(GogDownloadDialog, {
      open: true,
      onOpenChange: () => {},
      gogId: 1207658695,
      gameId: "sky",
    })
  );
}

test("confirmation explains Windows, ScummVM, destination and the separate optional patch", () => {
  const html = renderDialog();
  expect(html).toContain("only supported on Windows");
  expect(html).toContain("ScummVM");
  expect(html).toContain("Installation folder");
  expect(html).toContain("Browse");
  expect(html).toContain("Next");
  expect(html).toContain("Installing the Hebrew patch is optional");
});

test("download and completion screens show progress then the library action", () => {
  useGogDownload.setState({ gameId: "sky", busy: true, step: "Downloading game…", percentage: 42 });
  const downloading = renderDialog();
  expect(downloading).toContain('role="progressbar"');
  expect(downloading).toContain('aria-valuenow="42"');
  expect(downloading).toContain("42%");
  expect(downloading).toContain("Hide");
  expect(downloading).not.toContain(">Next<");
  useGogDownload.setState({ busy: false, complete: true, percentage: 100, step: "Ready to play" });
  const complete = renderDialog();
  expect(complete).toContain("Game installed");
  expect(complete).toContain("View library");
  expect(complete).toContain("ready to play through ScummVM");
});

test("GOG installation requires confirmed ownership even for direct start calls", async () => {
  for (const account of [
    { gogAuth: { isAuthenticated: false }, gogGames: [{ id: 1, title: "Stale owned game" }] },
    { gogAuth: { isAuthenticated: true }, gogGames: [] },
    { gogAuth: { isAuthenticated: true }, gogGames: [{ id: 2, title: "Different game" }] },
  ]) {
    useAppStore.setState(account);
    await useGogDownload.getState().start(1, "monkey2-se", "/Games");
    expect(invoke).not.toHaveBeenCalled();
    expect(useGogDownload.getState().busy).toBe(false);
    expect(useGogDownload.getState().error).toContain("owns this game");
  }
});

test("an open GOG dialog disables installation when ownership is absent", () => {
  useAppStore.setState({ gogGames: [] });
  const html = renderDialog();
  expect(html).toContain("Connect the GOG account that owns this game");
  expect(html).toMatch(/<button[^>]*disabled[^>]*>Next<\/button>/);
});

const { useSteamDownload } = await import("../src/store/steamDownload");
test("Steam replacement requires confirmation and cancellation never overwrites", async () => {
  invoke.mockImplementation((command) =>
    command === "download_steam_windows_game"
      ? Promise.reject("OVERWRITE_REQUIRED:/Games/steam-10110")
      : Promise.resolve([])
  );
  ask.mockResolvedValue(false);
  await useSteamDownload.getState().start(10110, ["sq1"], "/Games");
  expect(ask).toHaveBeenCalled();
  expect(
    invoke.mock.calls.filter(([command]) => command === "download_steam_windows_game")
  ).toHaveLength(1);
  expect(useSteamDownload.getState().error).toBeNull();
});

test("Steam replacement retries with overwrite only after confirmation", async () => {
  invoke.mockImplementation((command, args) =>
    command === "download_steam_windows_game" && !(args as { overwrite: boolean }).overwrite
      ? Promise.reject("OVERWRITE_REQUIRED:/Games/steam-10110")
      : Promise.resolve([])
  );
  ask.mockResolvedValue(true);
  await useSteamDownload.getState().start(10110, ["sq1"], "/Games");
  expect(invoke).toHaveBeenCalledWith("download_steam_windows_game", {
    appId: 10110,
    gameIds: ["sq1"],
    installRoot: "/Games",
    overwrite: true,
  });
  expect(useSteamDownload.getState().error).toBeNull();
});
