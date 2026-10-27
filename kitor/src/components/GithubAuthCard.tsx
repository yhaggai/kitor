import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export default function GithubAuthCard() {
  const [configured, setConfigured] = useState<boolean | null>(null);
  const [token, setToken] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  useEffect(() => {
    invoke<boolean>("github_token_configured")
      .then(setConfigured)
      .catch((error) => setMessage(String(error)));
  }, []);
  const save = async () => {
    setBusy(true);
    setMessage("");
    try {
      await invoke("save_github_token", { token });
      setToken("");
      setConfigured(true);
      setMessage("Repository access verified. Token saved in your system credential store.");
    } catch (error) {
      setToken("");
      setMessage(String(error));
    } finally {
      setBusy(false);
    }
  };
  const remove = async () => {
    setBusy(true);
    try {
      await invoke("remove_github_token");
      setConfigured(false);
      setToken("");
      setMessage("Saved token removed from this device.");
    } catch (error) {
      setMessage(String(error));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Card>
      <CardHeader>
        <CardTitle>Private patch downloads</CardTitle>
        <CardDescription>
          Connect to yhaggai/kitor with a fine-grained GitHub token with Contents: read-only access.
          Kitor stores it securely on this device and reuses it after restarting. GitHub CLI is not
          required.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-sm">
          {configured === null
            ? "Checking saved token…"
            : configured
              ? "Token saved"
              : "No token saved — public downloads only"}
        </p>
        <label className="block text-sm font-medium" htmlFor="github-download-token">
          {configured ? "Replacement GitHub token" : "GitHub token"}
        </label>
        <input
          id="github-download-token"
          type="password"
          autoComplete="off"
          spellCheck={false}
          value={token}
          onChange={(event) => setToken(event.target.value)}
          disabled={busy}
          className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
          placeholder="Paste your token here"
        />
        <div className="flex gap-2">
          <Button onClick={save} disabled={busy || !token.trim()}>
            {busy ? "Working…" : "Verify and Save"}
          </Button>
          {configured && (
            <Button variant="outline" onClick={remove} disabled={busy}>
              Remove Saved Token
            </Button>
          )}
        </div>
        <p className="text-xs text-muted-foreground">
          Access lasts until the token expires or is revoked. Tokens are never included in patch
          files or logs.
        </p>
        {message && (
          <p aria-live="polite" className="text-sm break-words">
            {message}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
