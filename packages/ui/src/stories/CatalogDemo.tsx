import { Download, FolderOpen, type LucideIcon, RefreshCw, RotateCcw, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import {
  AppShell,
  Button,
  type Collection,
  CollectionView,
  GameDetailsView,
  type GameSummary,
  ManageMenu,
  type SettingsState,
  SettingsView,
  useGameNavigation,
} from "../index";
import { games as fixtures } from "../mocks/games";
import type { ReviewScenario } from "../mocks/operations";
import { type FlowRequest, ReviewFlow } from "./ReviewFlow";

const descriptions: Record<string, string> = {
  restore:
    "Restore the original files from backup and remove the current translation. Restore first before changing translation editions.",
  remove: "Remove this game from your library. Game files on disk will not be deleted.",
  "remove-all": "Remove all games from your library. Game files on disk will not be deleted.",
  update:
    "Installed version 1.0 → available version 1.1. Close the game before continuing. Release notes: corrected dialogue and improved compatibility.",
  install:
    "Download the selected translation, back up the original files and apply the patch. Close the game before continuing.",
  scan: "Automatically find recognized games in Steam and GOG folders. Existing games are skipped.",
  folder:
    "Choose an installed game folder. Kitor adds it only if it matches a game in the translation registry.",
  "install-runtime":
    "Kitor manages its own ScummVM installation. After setup, you can launch this game.",
  "remove-runtime":
    "Uninstall Kitor’s ScummVM runtime. Games that use it will need setup again before they can launch.",
  "install-app":
    "Kitor 1.1 · Improved game detection and translation reliability. Finish game downloads and patch changes first. Your library and settings will be kept.",
};

/** Review-only controller. Every asynchronous operation is intercepted by MSW. */
export function CatalogDemo({
  initialCollection = "library",
  initialGame,
  initialSettings = false,
  scenario = "normal",
  initialFlow,
  disconnected = false,
}: {
  initialCollection?: Collection;
  initialGame?: string;
  initialSettings?: boolean;
  scenario?: ReviewScenario;
  initialFlow?: string;
  disconnected?: boolean;
}) {
  const [collection, setCollection] = useState(initialCollection);
  const [settingsOpen, setSettingsOpen] = useState(initialSettings);
  const [settings, setSettings] = useState<SettingsState>({
    steam: !disconnected,
    gog: !disconnected,
    steamKey: !disconnected,
    runtime: scenario !== "missing-runtime",
    runtimeUpdated: false,
    appUpdated: false,
  });
  const [games, setGames] = useState<GameSummary[]>([]);
  const [changes, setChanges] = useState<Record<string, Partial<GameSummary>>>({});
  const [removed, setRemoved] = useState<string[]>([]);
  const [added, setAdded] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>();
  const [attempt, setAttempt] = useState(0);
  const [notice, setNotice] = useState("");
  const [warnings, setWarnings] = useState(false);
  const [updateAvailable, setUpdateAvailable] = useState(true);
  const [appAvailable, setAppAvailable] = useState(false);
  const [restartReady, setRestartReady] = useState(false);
  const [choice, setChoice] = useState("Hebrew");
  const [busy, setBusy] = useState(false);
  const [flow, setFlow] = useState<FlowRequest | undefined>(
    initialFlow
      ? {
          action: initialFlow,
          title: titles[initialFlow] ?? initialFlow,
          description: descriptions[initialFlow],
          gameId: initialGame,
        }
      : undefined
  );
  const navigation = useGameNavigation();
  const select = navigation.select;
  useEffect(() => {
    if (initialGame) select(initialGame);
  }, [initialGame, select]);
  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError(undefined);
    fetch(`/api/catalog/${collection}?attempt=${attempt}`, { signal: controller.signal })
      .then(async (response) => {
        if (!response.ok)
          throw new Error("The catalog is temporarily unavailable. Please try again.");
        return response.json() as Promise<GameSummary[]>;
      })
      .then((items) => {
        setGames(items);
        setLoading(false);
      })
      .catch((reason: Error) => {
        if (!controller.signal.aborted) {
          setError(reason.message);
          setLoading(false);
        }
      });
    return () => controller.abort();
  }, [collection, attempt]);
  const all = [
    ...games,
    ...fixtures.filter(
      (item) => added.includes(item.id) && !games.some((game) => game.id === item.id)
    ),
  ].map((item) => ({
    ...item,
    ...changes[item.id],
    ...(collection === "discovery" && removed.includes(item.id)
      ? { status: "unowned" as const }
      : {}),
  }));
  const visible = all.filter((item) => collection === "discovery" || !removed.includes(item.id));
  const game = visible.find((item) => item.id === navigation.selectedId);
  function open(action: string, title = titles[action] ?? action) {
    if (busy) {
      setNotice("Finish the current operation before starting another.");
      return;
    }
    setFlow({ action, title, description: descriptions[action], gameId: game?.id, choice });
  }
  function finish(request: FlowRequest, message: string) {
    setNotice(message);
    const id = request.gameId;
    const patch = (value: Partial<GameSummary>) => {
      if (id) setChanges((prev) => ({ ...prev, [id]: { ...prev[id], ...value } }));
    };
    switch (request.action) {
      case "install":
      case "update":
        patch({
          status: "ready",
          version: request.action === "update" ? "1.1" : "1.0",
          language: request.choice ?? "Hebrew",
        });
        setUpdateAvailable(false);
        break;
      case "restore":
        patch({ status: "available" });
        break;
      case "refresh":
        if (game?.status === "attention") {
          setNotice(
            "Game files missing · locate or reinstall the game, then refresh. Patch could not be verified."
          );
        }
        break;
      case "remove":
        if (id) setRemoved((prev) => [...prev, id]);
        navigation.back();
        break;
      case "remove-all":
        setRemoved(fixtures.map((item) => item.id));
        navigation.back();
        break;
      case "steam":
      case "gog":
        if (id) {
          setAdded((prev) => [...prev, id]);
          setRemoved((prev) => prev.filter((item) => item !== id));
          patch({ status: "available" });
        }
        break;
      case "scan":
      case "folder": {
        const detected =
          request.action === "scan" ? "grim-fandango-remastered" : (request.gameId ?? "tentacle");
        setAdded((prev) => [...prev, detected]);
        setRemoved((prev) => prev.filter((item) => item !== detected));
        setChanges((prev) => ({ ...prev, [detected]: { ...prev[detected], status: "available" } }));
        setWarnings(scenario === "scan-warnings");
        break;
      }
      case "check-patches":
        setUpdateAvailable(true);
        break;
      case "connect-steam":
        setSettings((prev) => ({ ...prev, steam: true }));
        break;
      case "connect-gog":
        setSettings((prev) => ({ ...prev, gog: true }));
        break;
      case "disconnect-steam":
        setSettings((prev) => ({ ...prev, steam: false, steamKey: false }));
        break;
      case "disconnect-gog":
        setSettings((prev) => ({ ...prev, gog: false }));
        break;
      case "steam-key":
        setSettings((prev) => ({ ...prev, steamKey: true }));
        break;
      case "install-runtime":
      case "update-runtime":
        setSettings((prev) => ({ ...prev, runtime: true, runtimeUpdated: true }));
        break;
      case "remove-runtime":
        setSettings((prev) => ({ ...prev, runtime: false }));
        break;
      case "check-app":
        setAppAvailable(scenario !== "up-to-date");
        break;
      case "install-app":
        setRestartReady(true);
        setAppAvailable(false);
        break;
      case "restart":
        setRestartReady(false);
        setSettings((prev) => ({ ...prev, appUpdated: true }));
        break;
    }
  }
  const actionButton = (
    action: string,
    label = titles[action] ?? action,
    disabled = false,
    Icon?: LucideIcon
  ) => (
    <Button
      key={action}
      variant="secondary"
      disabled={busy || disabled}
      onClick={() => open(action)}
    >
      {Icon && <Icon size={18} aria-hidden="true" className="kt-action-icon" />}
      {label}
    </Button>
  );
  const installed = game?.status === "ready";
  const owned = game?.status !== "unowned";
  const unhealthy = game?.status === "attention";
  const translation = game && (
    <div className="kt-flow-body">
      <label className="kt-field">
        Translation edition
        <select value={choice} disabled={busy} onChange={(event) => setChoice(event.target.value)}>
          <option>Hebrew</option>
          <option>Hebrew — alternate edition</option>
        </select>
      </label>
      <p>
        Available: {updateAvailable && game.id === "jenny-leclue" ? "1.1" : "1.0"} ·{" "}
        {installed
          ? `Installed: ${game.language ?? "Hebrew"} ${game.version ?? "1.0"}`
          : "No active translation"}
      </p>
      {installed && choice !== (game.language ?? "Hebrew") ? (
        <>
          <p>Restore the current translation before installing another edition.</p>
          {actionButton("restore", "Restore to change translation")}
        </>
      ) : installed ? (
        updateAvailable && game.id === "jenny-leclue" ? (
          actionButton("update", "Install translation update")
        ) : (
          <Button disabled>Translation installed</Button>
        )
      ) : owned && !unhealthy ? (
        actionButton("install", "Install translation")
      ) : (
        <p>
          {unhealthy
            ? "Refresh and repair missing game files before installing."
            : "Install the base game first."}
        </p>
      )}
    </div>
  );
  return (
    <AppShell
      active={settingsOpen ? "settings" : collection}
      onNavigate={(next) => {
        setSettingsOpen(next === "settings");
        if (next !== "settings") {
          navigation.back();
          setCollection(next);
        }
      }}
      footer={
        <>
          <Button variant="ghost" onClick={() => open("report")}>
            Report a bug
          </Button>
          <p className="kt-review-note">Interactive preview · sample data</p>
        </>
      }
    >
      <div ref={navigation.root}>
        {notice && <output className="kt-notice">{notice}</output>}
        {warnings && (
          <details className="kt-notice">
            <summary>Scan warnings (1)</summary>
            <p>
              One GOG folder was inaccessible. Other folders were scanned successfully. Choose that
              folder manually to retry.
            </p>
          </details>
        )}
        {flow && (
          <ReviewFlow
            key={`${flow.action}-${flow.gameId}`}
            request={flow}
            scenario={scenario}
            connectedGog={settings.gog}
            onDone={finish}
            onClose={() => setFlow(undefined)}
            onBusy={setBusy}
          />
        )}
        {settingsOpen ? (
          <>
            <SettingsView state={settings} onAction={open} />
            {appAvailable && (
              <div className="kt-notice">
                <h2>Kitor 1.1 is available</h2>
                <p>Improved game detection and translation reliability.</p>
                {actionButton("install-app", "Update and restart")}
              </div>
            )}
            {restartReady && (
              <div className="kt-notice">
                Kitor is ready to restart. {actionButton("restart", "Restart Kitor")}
              </div>
            )}
          </>
        ) : (
          <>
            <div hidden={!!game}>
              <CollectionView
                key={collection}
                collection={collection}
                games={visible}
                loading={loading}
                error={error}
                onRetry={() => setAttempt((value) => value + 1)}
                onOpen={navigation.open}
                toolbar={
                  <ManageMenu
                    key={collection}
                    label={collection === "library" ? "Manage library" : "Manage discovery"}
                  >
                    <div className="kt-flow-actions">
                      {actionButton("scan", "Scan Steam & GOG")}
                      {actionButton("folder", "Add game from folder")}
                      {collection === "library" && (
                        <>
                          {actionButton("check-patches", "Check translation updates")}
                          {actionButton("remove-all", "Remove all games", !visible.length)}
                        </>
                      )}
                    </div>
                  </ManageMenu>
                }
              />
            </div>
            {game && (
              <GameDetailsView
                key={game.id}
                game={game}
                collection={collection}
                onBack={navigation.back}
                translation={translation}
                actions={
                  owned ? (
                    <>
                      <Button
                        disabled={busy || unhealthy}
                        onClick={() =>
                          open(
                            !settings.runtime && game.id !== "jenny-leclue"
                              ? "install-runtime"
                              : "play"
                          )
                        }
                      >
                        {installed ? "Play in Hebrew" : "Play original"}
                      </Button>
                      {!installed && !unhealthy && actionButton("install", "Install translation")}
                    </>
                  ) : (
                    <>
                      <Button disabled={busy} onClick={() => open("store-steam", "Get the game")}>
                        Get the game
                      </Button>
                      {actionButton("steam", "Install from Steam")}
                      {actionButton("gog", "Install from GOG")}
                      {actionButton("store-gog", "View on GOG")}
                    </>
                  )
                }
                manage={
                  <div className="kt-flow-body">
                    <h2>Manage game</h2>
                    {unhealthy && (
                      <p>
                        Game files missing · patch could not be verified. Reinstall or locate the
                        game, then refresh.
                      </p>
                    )}
                    {!owned && <p>This game isn’t installed yet.</p>}
                    <div className="kt-flow-actions">
                      {owned ? (
                        <>
                          {actionButton("open-folder", "Open game folder", false, FolderOpen)}
                          {actionButton("refresh", "Refresh game", false, RefreshCw)}
                          {installed &&
                            actionButton("restore", "Restore original game", false, RotateCcw)}
                          {actionButton("remove", "Remove game", false, Trash2)}
                          {unhealthy &&
                            actionButton("folder", "Locate game folder", false, FolderOpen)}
                        </>
                      ) : (
                        <>
                          {actionButton("steam", "Install from Steam", false, Download)}
                          {actionButton("gog", "Install from GOG", false, Download)}
                        </>
                      )}
                    </div>
                  </div>
                }
              />
            )}
          </>
        )}
      </div>
    </AppShell>
  );
}
const titles: Record<string, string> = {
  install: "Install translation",
  update: "Update translation",
  restore: "Restore original game",
  remove: "Remove game",
  "remove-all": "Remove all games",
  play: "Launch game",
  refresh: "Refresh game",
  "open-folder": "Open game folder",
  scan: "Scan for games",
  folder: "Add game from folder",
  steam: "Install game from Steam",
  gog: "Install game from GOG",
  "check-patches": "Check translation updates",
  "connect-steam": "Connect Steam",
  "connect-gog": "Connect GOG",
  "disconnect-steam": "Disconnect Steam",
  "disconnect-gog": "Log out of GOG",
  "refresh-steam": "Refresh Steam library",
  "refresh-gog": "Refresh GOG library",
  "steam-key": "Steam API key",
  "install-runtime": "Download ScummVM",
  "update-runtime": "Update ScummVM",
  "remove-runtime": "Uninstall ScummVM",
  "open-runtime": "Open ScummVM",
  "runtime-folder": "Open ScummVM folder",
  "check-app": "Check for app updates",
  "install-app": "Update Kitor",
  restart: "Restart Kitor",
  report: "Report a bug",
  "steam-privacy": "Steam privacy settings",
  "steam-api": "Get a Steam API key",
  "store-steam": "View on Steam",
  "store-gog": "View on GOG",
};
