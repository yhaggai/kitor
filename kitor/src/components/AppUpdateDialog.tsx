import { Download, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import { isInstallingUpdate } from "@/lib/appUpdates";
import { useAppUpdates } from "@/store/appUpdates";

export default function AppUpdateDialog() {
  const state = useAppUpdates();
  const busy = isInstallingUpdate(state.status);
  const percentage =
    state.total && state.total > 0
      ? Math.min(100, Math.round((state.downloaded / state.total) * 100))
      : null;
  const title =
    state.status === "upToDate"
      ? "Kitor is up to date"
      : state.status === "ready"
        ? "Kitor is ready to restart"
        : state.update
          ? `Kitor ${state.update.version} is available`
          : "Kitor updates";
  const progressText =
    state.status === "downloading"
      ? percentage === null
        ? `Downloading update… ${(state.downloaded / 1048576).toFixed(1)} MB`
        : `Downloading update… ${percentage}%`
      : state.status === "verifying"
        ? "Verifying the update…"
        : state.status === "installing"
          ? "Installing the update…"
          : "Restarting Kitor…";

  return (
    <Dialog open={state.open} onOpenChange={state.setOpen}>
      <DialogContent className={`text-foreground ${busy ? "[&>button]:hidden" : ""}`}>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>
            {state.currentVersion ? `Installed version: ${state.currentVersion}. ` : ""}
            {state.update
              ? "Install the latest version and restart Kitor."
              : "Check for the latest version of Kitor."}
          </DialogDescription>
        </DialogHeader>
        {state.status === "checking" && <output>Checking for updates…</output>}
        {state.update?.notes && (
          <div className="max-h-60 overflow-y-auto rounded-md bg-muted p-4">
            <h3 className="mb-2 text-sm font-semibold">What’s new</h3>
            <p className="whitespace-pre-wrap break-words text-sm text-muted-foreground">
              {state.update.notes}
            </p>
          </div>
        )}
        {state.status === "available" && (
          <p className="text-sm text-muted-foreground">
            {state.canInstall
              ? "Finish any game downloads or patch changes before updating. Your library and settings will be kept."
              : "You’re running a development build. Install a release build to use app updates."}
          </p>
        )}
        {busy && (
          <div className="space-y-2" aria-live="polite">
            <p className="text-sm">{progressText}</p>
            <Progress
              value={state.status === "downloading" ? percentage : null}
              aria-label="App update progress"
            />
          </div>
        )}
        {state.error && (
          <p role="alert" className="text-sm text-destructive">
            {state.error}
          </p>
        )}
        <DialogFooter>
          {!busy && (
            <Button variant="outline" onClick={() => state.setOpen(false)}>
              {state.status === "available" ? "Later" : "Close"}
            </Button>
          )}
          {state.status === "available" && (
            <Button disabled={!state.canInstall} onClick={() => void state.install()}>
              <Download className="mr-2 h-4 w-4" />
              Update and restart
            </Button>
          )}
          {state.status === "error" && (
            <Button onClick={() => void state.check(true)}>Try again</Button>
          )}
          {state.status === "ready" && (
            <Button onClick={() => void state.restart()}>
              <RefreshCw className="mr-2 h-4 w-4" />
              Restart Kitor
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
