import { BrandLogo, ManageMenu, useTranslation } from "@kitor/ui";
import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { isInstallingUpdate } from "@/lib/appUpdates";
import { useAppUpdates } from "@/store/appUpdates";

export default function AppUpdateCard() {
  const { t } = useTranslation();
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
        <CardTitle className="flex items-center gap-3">
          <span className="w-16">
            <BrandLogo />
          </span>
          {t("Kitor updates")}{" "}
        </CardTitle>
        <CardDescription>
          {state.currentVersion || version
            ? t("Version {0}", [state.currentVersion || version || "—"])
            : t("App updates")}{" "}
          {t("· Checked automatically when Kitor opens")}{" "}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm text-muted-foreground" aria-live="polite">
          {state.status === "checking"
            ? t("Checking for updates…")
            : state.status === "upToDate"
              ? t("You’re running the latest version.")
              : state.status === "error"
                ? state.error
                : state.update
                  ? t("Version {0} is available.", [state.update.version])
                  : t("Get the latest features and fixes.")}
        </p>
        <ManageMenu label={t("Manage Kitor updates")}>
          {state.update || isInstallingUpdate(state.status) ? (
            <Button onClick={() => state.setOpen(true)}>{t("View update")}</Button>
          ) : (
            <Button
              variant="outline"
              disabled={state.status === "checking"}
              onClick={() => void state.check(true)}
            >
              {t("Check for updates")}{" "}
            </Button>
          )}
        </ManageMenu>
      </CardContent>
    </Card>
  );
}
