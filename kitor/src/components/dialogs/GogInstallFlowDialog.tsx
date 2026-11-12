import { useTranslation } from "@kitor/ui";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { CheckCircle2, Circle, Loader2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { PatchInstallProgress } from "@/components/dialogs/InstallPatchDialog";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { useGameCardState } from "@/hooks/useGameCardState";
import { useGogLibrary } from "@/store/gog";
import { useGogDownload } from "@/store/gogDownload";
import { useGogInstallFlow } from "@/store/gogInstallFlow";
import { useAppStore } from "@/store/store";

export function GogInstallFlowDialog() {
  const { t, language } = useTranslation();
  const { target, open, setOpen } = useGogInstallFlow();
  const download = useGogDownload();
  const authenticated = useAppStore((s) => s.gogAuth.isAuthenticated);
  const games = useAppStore((s) => s.gogGames);
  const library = useGogLibrary();
  const [checked, setChecked] = useState<string | null>(null);
  const [loggingIn, setLoggingIn] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [capability, setSupported] = useState<{ gameId: string; value: boolean } | null>(null);
  const [launching, setLaunching] = useState(false);
  const steam = target?.steamAppId !== undefined;
  const active = target?.gameId === download.gameId;
  const installing = active && (download.busy || download.complete || download.error !== null);
  const patching = active && download.busy && download.phase === "patch";
  const { state: patchState, actions: patchActions } = useGameCardState();

  useEffect(() => {
    if (patching) patchActions.startInstall();
    else patchActions.finishInstall();
  }, [patching, patchActions]);

  useEffect(() => {
    if (patching) patchActions.updateProgress(download.percentage ?? 0, download.step);
  }, [patching, download.percentage, download.step, patchActions]);

  const supported = capability?.gameId === target?.gameId ? capability?.value : null;
  const owned = authenticated && games.some((game) => game.id === target?.gogId);

  const refresh = useCallback(async () => {
    setChecked(null);
    setError(null);
    await useGogLibrary.getState().refresh();
    setChecked(target?.gameId ?? null);
  }, [target?.gameId]);

  useEffect(() => {
    if (!open || !target || target.steamAppId !== undefined) return;
    let alive = true;
    setSupported(null);
    setError(null);
    setLoggingIn(false);
    void refresh();
    void invoke<boolean>("get_gog_install_capabilities", {
      gameId: target.gogId,
      scummvmId: target.gameId,
    })
      .then((value) => {
        if (alive) setSupported({ gameId: target.gameId, value });
      })
      .catch((err) => {
        if (alive) setError(String(err));
      });
    return () => {
      alive = false;
    };
  }, [open, target, refresh]);

  useEffect(() => {
    const success = listen("gog-login-success", () => {
      setLoggingIn(false);
      if (open && !steam) queueMicrotask(() => void refresh());
    });
    const failure = listen<string>("gog-login-error", ({ payload }) => {
      setLoggingIn(false);
      if (open && !steam) setError(payload);
    });
    return () => {
      void success.then((stop) => stop());
      void failure.then((stop) => stop());
    };
  }, [open, refresh, steam]);

  const start = useCallback(async () => {
    if (!target || useGogDownload.getState().busy) return;
    setError(null);
    try {
      const previous = useGogDownload.getState();
      const installedGameId =
        target.installedGameId ??
        (previous.gameId === target.gameId ? previous.installedGameId : null);
      if (installedGameId !== null) {
        await previous.start({ installedGameId }, target.gameId);
      } else if (target.steamAppId !== undefined) {
        await useGogDownload
          .getState()
          .start({ steamAppId: target.steamAppId }, target.gameId, target.installRoot);
      } else {
        await useGogDownload.getState().start(target.gogId, target.gameId);
      }
    } catch (err) {
      setError(String(err));
    }
  }, [target]);

  useEffect(() => {
    if (
      open &&
      target &&
      (steam ||
        (checked === target.gameId && !library.loading && !library.error && owned && supported)) &&
      !error &&
      !installing &&
      !download.busy
    ) {
      void start();
    }
  }, [
    open,
    target,
    steam,
    checked,
    library.loading,
    library.error,
    error,
    owned,
    supported,
    installing,
    download.busy,
    start,
  ]);

  async function login() {
    setLoggingIn(true);
    setError(null);
    try {
      await invoke("login_to_gog");
    } catch (err) {
      setError(String(err));
      setLoggingIn(false);
    }
  }
  async function buy() {
    if (!target || target.steamAppId !== undefined) return;
    setError(null);
    try {
      if (target.purchaseUrl) await openUrl(target.purchaseUrl);
      else await openUrl(`https://www.gog.com/checkout/manual/${target.gogId}`);
    } catch (err) {
      setError(String(err));
    }
  }
  async function play() {
    if (download.installedGameId === null) return;
    setLaunching(true);
    setError(null);
    try {
      await invoke("launch_game", { gameId: download.installedGameId });
      useGogInstallFlow.setState({ open: false, target: null });
    } catch (err) {
      setError(String(err));
    } finally {
      setLaunching(false);
    }
  }

  if (!target) return null;
  const steps = ["Downloading the game", "Installing the game", "Installing the Hebrew patch"];
  const index = !installing
    ? 0
    : download.phase === "patch"
      ? 2
      : download.phase === "install"
        ? 1
        : 0;
  const complete = active && download.complete;
  const failure =
    error || (active ? download.error : null) || (!steam && !installing ? library.error : null);
  const checking =
    checked !== target.gameId || library.loading || (owned && supported === null && !error);

  const preparing = (steam || (owned && supported !== false)) && !installing && !failure;
  const showSteps = installing || preparing;
  const progressing = preparing || (active && download.busy);
  const percentage = installing ? download.percentage : null;
  const downloadedMB =
    installing && download.phase === "download" && download.downloadedBytes !== null
      ? new Intl.NumberFormat(language, { maximumFractionDigits: 1 }).format(
          download.downloadedBytes / 1_000_000
        )
      : null;

  const totalMB =
    download.totalBytes !== null && download.totalBytes > 0
      ? new Intl.NumberFormat(language, { maximumFractionDigits: 1 }).format(
          download.totalBytes / 1_000_000
        )
      : null;

  if (!open && installing) {
    return (
      <Button className="fixed bottom-6 end-6 z-40 shadow-lg" onClick={() => setOpen(true)}>
        {t("Game installation")}
      </Button>
    );
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="gog-installer" dir={language === "he" ? "rtl" : "ltr"}>
        {showSteps && (
          <ol className="gog-installer-steps" aria-label={t("Game installation")}>
            {steps.map((step, i) => {
              const done = complete || i < index;
              const current = !complete && i === index;
              const Icon = done ? CheckCircle2 : current && progressing ? Loader2 : Circle;
              return (
                <li
                  key={step}
                  aria-current={current ? "step" : undefined}
                  className={done ? "is-complete" : current ? "is-current" : ""}
                >
                  <Icon
                    aria-hidden="true"
                    className={current && progressing ? "animate-spin" : ""}
                  />
                  <span>{t(step)}</span>
                </li>
              );
            })}
          </ol>
        )}
        <div className="gog-installer-main">
          <div className="space-y-4" aria-live="polite">
            {complete && !failure && (
              <CheckCircle2 className="h-9 w-9 text-green-500" aria-hidden="true" />
            )}
            <DialogTitle>
              {complete
                ? t(failure ? "Could not start the game" : "Ready to play")
                : showSteps
                  ? t(steps[index])
                  : t("Play with GOG")}
            </DialogTitle>
            <DialogDescription>
              {complete
                ? t(
                    failure
                      ? "The game is installed, but could not be started. You can retry without reinstalling."
                      : "The game and Hebrew patch are installed. Enjoy your game!"
                  )
                : installing
                  ? t("Keep Kitor open until installation finishes.")
                  : steam
                    ? t("Preparing your installation…")
                    : checking
                      ? t("Checking your GOG library…")
                      : !authenticated
                        ? t("Connect your GOG account to install and play this game.")
                        : !owned
                          ? t("Buy the game on GOG, then return here to continue.")
                          : supported === false
                            ? t(
                                "Automatic installation is not available for this game on this computer. Install it from GOG, then add its folder to Kitor."
                              )
                            : t("Preparing your installation…")}
            </DialogDescription>
            {patching && (
              <PatchInstallProgress
                installSteps={patchState.installSteps}
                progress={patchState.installProgress}
                download={download.patchDownload}
              />
            )}
            {progressing && !patching && (
              <div
                role="progressbar"
                aria-label={t(steps[index])}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={percentage ?? undefined}
                className="gog-installer-progress"
              >
                <div
                  className={percentage === null ? "steam-download-indicator" : ""}
                  style={{
                    width: percentage === null ? "33%" : `${percentage}%`,
                  }}
                />
              </div>
            )}
            {progressing && !patching && (percentage !== null || downloadedMB !== null) && (
              <p className="text-sm tabular-nums" dir="ltr">
                {percentage !== null && `${Math.floor(percentage)}%`}
                {percentage !== null && downloadedMB !== null && " · "}
                {downloadedMB !== null &&
                  `${downloadedMB}${totalMB !== null ? ` / ${totalMB}` : ""} MB`}
              </p>
            )}
            {failure && (
              <p role="alert" className="text-sm text-destructive">
                {t(failure)}
              </p>
            )}
          </div>
          <div className="gog-installer-actions">
            {complete ? (
              <Button onClick={() => void play()} disabled={launching}>
                {t("Play the game")}
              </Button>
            ) : download.busy ? (
              <Button variant="outline" onClick={() => setOpen(false)}>
                {t("Hide")}
              </Button>
            ) : installing ? (
              <Button onClick={() => void start()}>{t("Try again")}</Button>
            ) : steam ? (
              <Loader2 className="animate-spin" aria-label={t("Loading…")} />
            ) : checking ? (
              <Loader2 className="animate-spin" aria-label={t("Loading…")} />
            ) : library.error ? (
              <Button onClick={() => void refresh()}>{t("Try again")}</Button>
            ) : !authenticated ? (
              <>
                <Button onClick={() => void login()} disabled={loggingIn}>
                  {loggingIn ? t("Waiting for login...") : t("Login to GOG")}
                </Button>
                {loggingIn && (
                  <Button
                    variant="outline"
                    onClick={() => {
                      setLoggingIn(false);
                      void refresh();
                    }}
                  >
                    {t("Check again")}
                  </Button>
                )}
              </>
            ) : !owned ? (
              <>
                <Button onClick={() => void buy()}>{t("Buy with GOG")}</Button>
                <Button variant="outline" onClick={() => void refresh()}>
                  {t("I purchased the game")}
                </Button>
              </>
            ) : supported === false ? (
              <Button onClick={() => void buy()}>{t("Open GOG")}</Button>
            ) : failure ? (
              <Button onClick={() => setOpen(false)}>{t("Close")}</Button>
            ) : null}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
