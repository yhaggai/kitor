import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useSteamStore } from "@/store/steam";

export default function SteamAuthCard() {
  const { status, busy, error, run } = useSteamStore();
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
        <CardTitle>Steam Integration</CardTitle>
        <CardDescription>
          Connect Steam to find games you own, including games you haven’t installed.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Sign in on Steam’s website to share your Steam ID. Kitor never receives your password.
          Your game details must be public for library lookup; signing in does not grant access to a
          private library.
        </p>
        <p className="font-medium">
          {status.steam_id ? `Connected · ${status.steam_id}` : "Not connected"}
        </p>
        {status.owned_app_ids !== null && (
          <p className="text-sm">{status.owned_app_ids.length} games in your Steam library</p>
        )}
        {status.message && (
          <output className="block text-sm text-muted-foreground">{status.message}</output>
        )}
        <div className="flex flex-wrap gap-2">
          {!status.steam_id && (
            <Button disabled={busy} onClick={() => run("connect_steam")}>
              {status.connecting ? "Waiting for Steam…" : "Connect Steam"}
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
              {status.connecting ? "Cancel sign-in" : status.steam_id ? "Disconnect" : "Clear key"}
            </Button>
          )}
          {status.steam_id && (
            <Button
              variant="outline"
              disabled={busy || !status.has_api_key}
              onClick={() => run("refresh_steam_library")}
            >
              Refresh library
            </Button>
          )}
        </div>
        <form
          className="space-y-2 border-t pt-4"
          onSubmit={async (event) => {
            event.preventDefault();
            if (await run("set_steam_api_key", { apiKey: key })) setKey("");
          }}
        >
          <label htmlFor="steam-api-key" className="block text-sm font-medium">
            Your Steam Web API key
          </label>
          <p className="text-sm text-muted-foreground">
            {status.has_api_key ? "Key saved securely." : "Enter your own key to check ownership."}{" "}
            The key is saved in your system credential store and restored when Kitor starts. It is
            sent only to Steam. Disconnect removes the saved key and Steam ID.
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
            placeholder="32-character API key"
          />
          <div className="flex flex-wrap gap-2">
            <Button type="submit" disabled={busy || !key.trim()}>
              Save key
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => open("https://steamcommunity.com/dev/apikey")}
            >
              Get a Steam API key
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => open("https://steamcommunity.com/my/edit/settings")}
            >
              Steam privacy settings
            </Button>
          </div>
        </form>
        {(error || linkError) && (
          <p role="alert" className="text-sm text-destructive">
            {error || linkError}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
