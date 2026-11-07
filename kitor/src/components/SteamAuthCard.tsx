import { ManageMenu, steamIcon, useTranslation } from "@kitor/ui";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useSteamStore } from "@/store/steam";

export default function SteamAuthCard() {
  const { t } = useTranslation();
  const { status, busy, error, run } = useSteamStore();
  const [showKey, setShowKey] = useState(false);
  const [key, setKey] = useState("");
  const [linkError, setLinkError] = useState<string | null>(null);
  const open = async (url: string) => {
    setLinkError(null);
    try {
      await openUrl(url);
    } catch (e) {
      setLinkError(String(e));
    }
  };
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-3">
          <img className="h-10 w-10 object-contain" src={steamIcon} alt="" />
          Steam
        </CardTitle>
        <CardDescription>
          {t("Connect Steam to find games you own, including games you haven’t installed.")}{" "}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          {t(
            "Sign in on Steam’s website to share your Steam ID. Kitor never receives your password. Your game details must be public for library lookup; signing in does not grant access to a private library."
          )}{" "}
        </p>
        <p className="font-medium">
          {status.steam_id ? t("Connected · {0}", [status.steam_id]) : t("Not connected")}
        </p>
        {status.owned_app_ids !== null && (
          <p className="text-sm">
            {status.owned_app_ids.length} {t("games in your Steam library")}
          </p>
        )}
        {status.message && (
          <output className="block text-sm text-muted-foreground">{status.message}</output>
        )}
        <ManageMenu label={t("Manage Steam")}>
          {!status.steam_id && (
            <Button disabled={busy} onClick={() => run("connect_steam")}>
              {status.connecting ? t("Waiting for Steam…") : t("Connect Steam")}
            </Button>
          )}
          {(status.steam_id || status.has_api_key || status.connecting) && (
            <Button
              variant="outline"
              disabled={busy && !status.connecting}
              onClick={() => {
                setKey("");
                run("disconnect_steam");
              }}
            >
              {status.connecting
                ? t("Cancel sign-in")
                : status.steam_id
                  ? t("Disconnect")
                  : t("Clear key")}
            </Button>
          )}
          {status.steam_id && (
            <Button
              variant="outline"
              disabled={busy || !status.has_api_key}
              onClick={() => run("refresh_steam_library")}
            >
              {t("Refresh library")}{" "}
            </Button>
          )}
          <Button variant="outline" onClick={() => setShowKey(true)}>
            {t("Manage Steam API key")}{" "}
          </Button>
        </ManageMenu>
        {showKey && (
          <form
            className="space-y-2 border-t pt-4"
            onSubmit={async (event) => {
              event.preventDefault();
              if (await run("set_steam_api_key", { apiKey: key })) setKey("");
            }}
          >
            <label htmlFor="steam-api-key" className="block text-sm font-medium">
              {t("Your Steam Web API key")}{" "}
            </label>
            <p className="text-sm text-muted-foreground">
              {status.has_api_key
                ? t("Key saved securely.")
                : t("Enter your own key to check ownership.")}{" "}
              {t(
                "The key is saved in your system credential store and restored when Kitor starts. It is sent only to Steam. Disconnect removes the saved key and Steam ID."
              )}{" "}
            </p>
            <input
              id="steam-api-key"
              type="password"
              autoComplete="off"
              spellCheck={false}
              value={key}
              onChange={(event) => setKey(event.target.value)}
              disabled={busy}
              className="h-10 w-full rounded-md border border-input bg-background px-3 text-sm"
              placeholder={t("32-character API key")}
            />
            <div className="flex flex-wrap gap-2">
              <Button type="submit" disabled={busy || !key.trim()}>
                {t("Save key")}{" "}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => open("https://steamcommunity.com/dev/apikey")}
              >
                {t("Get a Steam API key")}{" "}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={() => open("https://steamcommunity.com/my/edit/settings")}
              >
                {t("Steam privacy settings")}{" "}
              </Button>
            </div>
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                setShowKey(false);
                setKey("");
              }}
            >
              {t("Close")}{" "}
            </Button>
          </form>
        )}
        {(error || linkError) && (
          <p role="alert" className="text-sm text-destructive">
            {error || linkError}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
