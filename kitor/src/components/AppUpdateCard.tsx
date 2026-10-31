import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { isInstallingUpdate } from "@/lib/appUpdates";
import { useAppUpdates } from "@/store/appUpdates";

export default function AppUpdateCard() {
  const state = useAppUpdates();
  const [version, setVersion] = useState<string | null>(null);
  useEffect(() => {
    void getVersion()
      .then(setVersion)
      .catch(() => {});
  }, []);
  return (
    <Card>
      <CardHeader>
        <CardTitle>Kitor updates</CardTitle>
        <CardDescription>
          {state.currentVersion || version
            ? `Version ${state.currentVersion || version}`
            : "App updates"}{" "}
          · Checked automatically when Kitor opens
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm text-muted-foreground" aria-live="polite">
          {state.status === "checking"
            ? "Checking for updates…"
            : state.status === "upToDate"
              ? "You’re running the latest version."
              : state.status === "error"
                ? state.error
                : state.update
                  ? `Version ${state.update.version} is available.`
                  : "Get the latest features and fixes."}
        </p>
        {state.update || isInstallingUpdate(state.status) ? (
          <Button onClick={() => state.setOpen(true)}>View update</Button>
        ) : (
          <Button
            variant="outline"
            disabled={state.status === "checking"}
            onClick={() => void state.check(true)}
          >
            Check for updates
          </Button>
        )}
      </CardContent>
    </Card>
  );
}
