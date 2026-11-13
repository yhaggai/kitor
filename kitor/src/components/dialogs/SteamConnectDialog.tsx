import { useTranslation } from "@kitor/ui";
import { Gamepad2, Loader2, Smartphone } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useNativeSteam } from "@/store/nativeSteam";

export default function SteamConnectDialog() {
  const { t, language } = useTranslation();
  const {
    open,
    busy,
    changing,
    method,
    qr,
    error,
    challenge,
    submittingCode,
    cancel,
    retry,
    switchMethod,
    submitCode,
  } = useNativeSteam();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [code, setCode] = useState("");
  // biome-ignore lint/correctness/useExhaustiveDependencies: Changing the sign-in method clears the current input.
  useEffect(() => {
    setPassword("");
    setCode("");
    if (!open) setUsername("");
  }, [open, method]);
  const field = "h-11 min-w-0 w-full rounded-md border border-input bg-background px-3 text-sm";
  return (
    <Dialog
      open={open}
      onOpenChange={(open) => {
        if (!open) void cancel();
      }}
    >
      <DialogContent
        dir={language === "he" ? "rtl" : "ltr"}
        className="max-h-[90vh] w-[calc(100vw-2rem)] min-w-0 grid-cols-[minmax(0,1fr)] overflow-y-auto sm:max-w-lg"
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-3">
            <Gamepad2 aria-hidden="true" className="h-9 w-9" />
            {t("Connect Steam")}
          </DialogTitle>
          <DialogDescription>
            {t("Sign in once to download your Steam games directly in Kitor.")}
          </DialogDescription>
        </DialogHeader>
        <fieldset className="grid grid-cols-2 gap-2" aria-label={t("Sign-in method")}>
          <Button
            variant={method === "qr" ? "default" : "outline"}
            aria-pressed={method === "qr"}
            disabled={changing}
            onClick={() => void switchMethod("qr")}
          >
            {t("QR code")}
          </Button>
          <Button
            variant={method === "password" ? "default" : "outline"}
            aria-pressed={method === "password"}
            disabled={changing}
            onClick={() => void switchMethod("password")}
          >
            {t("Username and password")}
          </Button>
        </fieldset>
        {method === "qr" ? (
          <>
            <div className="flex min-h-64 items-center justify-center rounded-xl bg-white p-4">
              {qr ? (
                <img
                  src={qr}
                  width={240}
                  height={240}
                  alt={t("Steam sign-in QR code")}
                  className="h-60 w-60"
                />
              ) : busy || changing ? (
                <Loader2
                  className="h-10 w-10 animate-spin text-slate-700"
                  aria-label={t("Creating a secure sign-in code…")}
                />
              ) : (
                <Smartphone className="h-12 w-12 text-slate-700" aria-hidden="true" />
              )}
            </div>
            <ol className="list-inside list-decimal space-y-2 text-sm">
              <li>{t("Open the Steam app on your phone.")}</li>
              <li>{t("Go to Steam Guard and scan this QR code.")}</li>
              <li>{t("Approve the Kitor sign-in. We’ll continue automatically.")}</li>
            </ol>
            {busy && (
              <output className="text-sm text-muted-foreground">
                {t(qr ? "Waiting for approval on your phone…" : "Creating a secure sign-in code…")}
              </output>
            )}
          </>
        ) : challenge ? (
          <div className="min-w-0 space-y-4">
            <output>{t(challenge.message)}</output>
            {challenge.code_type !== null && (
              <form
                className="space-y-3"
                onSubmit={(event) => {
                  event.preventDefault();
                  const submitted = code;
                  setCode("");
                  void submitCode(submitted);
                }}
              >
                <label htmlFor="steam-guard-code" className="block text-sm font-medium">
                  {t("Steam Guard code")}
                </label>
                <input
                  id="steam-guard-code"
                  autoComplete="one-time-code"
                  autoCapitalize="characters"
                  spellCheck={false}
                  dir="ltr"
                  maxLength={8}
                  value={code}
                  onChange={(event) => setCode(event.target.value)}
                  disabled={submittingCode}
                  className={field}
                />
                <Button type="submit" disabled={submittingCode || code.trim().length < 4}>
                  {t(submittingCode ? "Verifying…" : "Verify code")}
                </Button>
              </form>
            )}
          </div>
        ) : (
          <form
            className="min-w-0 space-y-4"
            onSubmit={(event) => {
              event.preventDefault();
              const submitted = password;
              setPassword("");
              void retry({ username: username.trim(), password: submitted });
            }}
          >
            <div className="space-y-2">
              <label htmlFor="steam-username" className="block text-sm font-medium">
                {t("Steam username")}
              </label>
              <input
                id="steam-username"
                name="username"
                autoComplete="username"
                autoCapitalize="none"
                spellCheck={false}
                dir="ltr"
                maxLength={128}
                value={username}
                onChange={(event) => setUsername(event.target.value)}
                disabled={busy || changing}
                className={field}
              />
            </div>
            <div className="space-y-2">
              <label htmlFor="steam-password" className="block text-sm font-medium">
                {t("Password")}
              </label>
              <input
                id="steam-password"
                name="password"
                type="password"
                autoComplete="current-password"
                dir="ltr"
                maxLength={1024}
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                disabled={busy || changing}
                className={field}
              />
            </div>
            <p className="text-sm text-muted-foreground">
              {t("Your password is sent securely to Steam and is not saved by Kitor.")}
            </p>
            <Button type="submit" disabled={busy || changing || !username.trim() || !password}>
              {busy && <Loader2 className="me-2 h-4 w-4 animate-spin" />}
              {t(busy ? "Signing in…" : "Sign in")}
            </Button>
          </form>
        )}
        {error && (
          <p
            role="alert"
            className="min-w-0 break-words text-sm text-destructive [overflow-wrap:anywhere]"
          >
            {t(error)}
          </p>
        )}
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={() => void cancel()}>
            {t("Cancel")}
          </Button>
          {method === "qr" && !busy && error && (
            <Button disabled={changing} onClick={() => void retry()}>
              {t("Generate a new code")}
            </Button>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
