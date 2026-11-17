import { useTranslation } from "@kitor/ui";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useId, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { type Contributor, useContributor } from "@/store/contributor";

function nameInitial(name: string): string {
  const trimmed = name.trim();
  if (!trimmed) return "?";
  return ([...trimmed][0] ?? "?").toLocaleUpperCase();
}

export function GoogleSignIn() {
  const { t } = useTranslation();
  const { pending, error, signIn, cancel } = useContributor();
  const disclosureId = useId();
  return (
    <div className="space-y-3">
      <p id={disclosureId}>
        {t(
          "Google verifies your account. Kitor’s contribution service stores your Google account ID, display name and verified email. If you submit edits, your display name and Kitor contributor ID appear on the public pull request. Signing in is optional."
        )}
      </p>
      {error && (
        <p role="alert" className="text-destructive">
          {t(error)}
        </p>
      )}
      {pending ? (
        <>
          <output>{t("Finish signing in in your browser…")}</output>
          <Button variant="outline" onClick={() => void cancel()}>
            {t("Cancel sign-in")}
          </Button>
        </>
      ) : (
        <Button aria-describedby={disclosureId} onClick={() => void signIn()}>
          {t("Sign in with Google")}
        </Button>
      )}
    </div>
  );
}

function ContributorProfile({ user }: { user: Contributor }) {
  const { t } = useTranslation();
  const signOut = useContributor((state) => state.signOut);
  const [enabled, setEnabled] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const descriptionId = useId();

  useEffect(() => {
    let active = true;
    invoke<boolean>("contributor_email_preference", { contributorId: user.id }).then(
      (value) => {
        if (active) setEnabled(value);
      },
      (reason: unknown) => {
        if (active) setError(String(reason));
      }
    );
    return () => {
      active = false;
    };
  }, [user.id]);

  async function requestPreference(value?: boolean) {
    setError("");
    setBusy(true);
    try {
      setEnabled(
        await invoke<boolean>("contributor_email_preference", {
          contributorId: user.id,
          enabled: value,
        })
      );
    } catch (reason) {
      setEnabled(null);
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <p dir="auto" className="font-medium">
        {user.name}
      </p>
      <div className="space-y-2 rounded-lg border p-4">
        <label className="flex items-center gap-3 font-medium">
          <input
            type="checkbox"
            role="switch"
            aria-checked={enabled === true}
            aria-describedby={descriptionId}
            checked={enabled === true}
            disabled={enabled === null || busy}
            onChange={(event) => void requestPreference(event.target.checked)}
          />
          {t("Allow contribution emails")}
        </label>
        <p id={descriptionId} className="text-sm text-muted-foreground">
          {t(
            "Receive approval and patch updates for contributions you opted in to. You can opt in separately when submitting a new contribution."
          )}
        </p>
        {error ? (
          <>
            <p role="alert" className="text-sm text-destructive">
              {t(error)}
            </p>
            <Button variant="outline" size="sm" onClick={() => void requestPreference()}>
              {t("Retry")}
            </Button>
          </>
        ) : (
          (enabled === null || busy) && (
            <output className="text-sm">
              {t(enabled === null ? "Loading email preference…" : "Saving email preference…")}
            </output>
          )
        )}
      </div>
      <Button variant="outline" className="justify-self-start" onClick={() => void signOut()}>
        {t("Sign out")}
      </Button>
    </>
  );
}

export default function ContributorAccount() {
  const { t } = useTranslation();
  const { user, pending, error, refresh, cancel } = useContributor();
  const [open, setOpen] = useState(false);
  const [profileOpen, setProfileOpen] = useState(false);
  useEffect(() => {
    void refresh();
  }, [refresh]);
  useEffect(() => {
    if (user) setOpen(false);
    else setProfileOpen(false);
  }, [user]);
  return (
    <div className="flex flex-wrap items-center justify-end gap-3 px-6 py-3">
      {user ? (
        <Dialog open={profileOpen} onOpenChange={setProfileOpen}>
          <DialogTrigger asChild>
            <Button variant="outline" size="sm">
              <span
                aria-hidden="true"
                className="inline-flex size-5 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-medium text-foreground"
              >
                {nameInitial(user.name)}
              </span>
              {t("Profile")}
            </Button>
          </DialogTrigger>
          <DialogContent aria-describedby={undefined}>
            <DialogTitle>{t("Contributor profile")}</DialogTitle>
            {profileOpen && <ContributorProfile key={user.id} user={user} />}
            {error && (
              <p role="alert" className="text-sm text-destructive">
                {t(error)}
              </p>
            )}
          </DialogContent>
        </Dialog>
      ) : (
        <Button variant="outline" size="sm" onClick={() => setOpen(true)}>
          {t("Sign in")}
        </Button>
      )}
      {error && !open && !profileOpen && (
        <span role="alert" className="text-sm text-destructive">
          {t(error)}
        </span>
      )}
      <Dialog
        open={open && !user}
        onOpenChange={(visible) => {
          setOpen(visible);
          if (!visible && pending) void cancel();
        }}
      >
        <DialogContent aria-describedby={undefined}>
          <DialogTitle>{t("Contributor sign-in")}</DialogTitle>
          <GoogleSignIn />
        </DialogContent>
      </Dialog>
    </div>
  );
}
