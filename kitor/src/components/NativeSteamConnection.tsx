import { ManageMenu, useTranslation } from "@kitor/ui";
import { CheckCircle2, Loader2 } from "lucide-react";
import { useEffect } from "react";
import { Button } from "@/components/ui/button";
import { useNativeSteam } from "@/store/nativeSteam";
import { useSteamStore } from "@/store/steam";

export default function NativeSteamConnection() {
  const { t } = useTranslation();
  const { connected, loading, busy, error, refresh, ensureConnected, disconnect } =
    useNativeSteam();
  const library = useSteamStore();
  useEffect(() => {
    void refresh();
  }, [refresh]);
  return (
    <div className="space-y-3">
      {loading ? (
        <output className="flex items-center gap-2">
          <Loader2 className="h-5 w-5 animate-spin" aria-hidden="true" />
          {t("Checking Steam connection…")}
        </output>
      ) : connected ? (
        <div className="flex items-start justify-between gap-3">
          <div>
            <p className="flex items-center gap-2 font-medium">
              <CheckCircle2 className="h-5 w-5 text-green-500" aria-hidden="true" />
              {t("Connected")}
            </p>
            <output className="text-sm text-muted-foreground">
              {library.busy
                ? t("Loading library…")
                : library.status.owned_app_ids !== null
                  ? t("Owned games: {0}", [library.status.owned_app_ids.length])
                  : library.error
                    ? t("Library could not be loaded")
                    : t("Refresh the library to see your owned games.")}
            </output>
          </div>
          <ManageMenu label={t("Manage Steam")}>
            <Button
              variant="outline"
              className="w-full"
              disabled={busy || library.busy}
              onClick={() => void library.run("refresh_steam_library")}
            >
              {t("Refresh library")}
            </Button>
            <Button
              variant="outline"
              className="w-full"
              disabled={busy}
              onClick={() => void disconnect()}
            >
              {t("Logout from Steam")}
            </Button>
            {library.error && (
              <p role="alert" className="break-words text-sm text-destructive">
                {t(library.error)}
              </p>
            )}
          </ManageMenu>
        </div>
      ) : (
        <Button disabled={busy} onClick={() => void ensureConnected()}>
          {t("Connect Steam")}
        </Button>
      )}
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
    </div>
  );
}
