import { ManageMenu, useTranslation } from "@kitor/ui";
import { CheckCircle2, Loader2 } from "lucide-react";
import { useEffect } from "react";
import { LibraryRefreshPreference } from "@/components/LibraryRefreshPreference";
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
      <div className="flex items-center gap-2">
        {loading ? (
          <Loader2 className="h-5 w-5 animate-spin" />
        ) : connected ? (
          <CheckCircle2 className="h-5 w-5 text-green-500" />
        ) : null}
        <p className="font-medium">
          {t(
            loading
              ? "Checking Steam connection…"
              : connected
                ? "Connected to Steam"
                : "Connect Steam"
          )}
        </p>
      </div>
      <p className="text-sm text-muted-foreground">
        {t(
          connected
            ? "You’re ready to download supported Steam games in Kitor."
            : "Kitor signs in to Steam to read your owned games and enable downloads. You can disconnect in Settings."
        )}
      </p>
      <Button
        variant={connected ? "outline" : "default"}
        disabled={busy || loading}
        onClick={() => {
          if (connected) void disconnect();
          else void ensureConnected();
        }}
      >
        {t(connected ? "Logout from Steam" : "Connect Steam")}
      </Button>
      {connected && (
        <div className="space-y-2">
          <LibraryRefreshPreference provider="steam" />
          <output className="text-sm text-muted-foreground">
            {library.busy
              ? t("Loading library…")
              : library.status.owned_app_ids !== null
                ? t("Owned games: {0}", [library.status.owned_app_ids.length])
                : library.error
                  ? t("Library could not be loaded")
                  : t("Refresh the library to see your owned games.")}
          </output>
          <ManageMenu label={t("Manage Steam")}>
            <Button
              variant="outline"
              className="w-full"
              disabled={busy || library.busy}
              onClick={() => void library.run("refresh_steam_library")}
            >
              {t("Refresh library")}
            </Button>
            {library.error && (
              <p role="alert" className="break-words text-sm text-destructive">
                {t(library.error)}
              </p>
            )}
          </ManageMenu>
        </div>
      )}
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
    </div>
  );
}
