import { useTranslation } from "@kitor/ui";
import { UserRound } from "lucide-react";
import { useEffect, useId, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { useContributor } from "@/store/contributor";

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

export default function ContributorAccount() {
  const { t } = useTranslation();
  const { user, pending, error, refresh, signOut, cancel } = useContributor();
  const [open, setOpen] = useState(false);
  useEffect(() => {
    void refresh();
  }, [refresh]);
  useEffect(() => {
    if (user) setOpen(false);
  }, [user]);
  return (
    <div className="flex flex-wrap items-center justify-end gap-3 px-6 py-3">
      {user ? (
        <>
          <span className="flex items-center gap-2">
            <UserRound size={18} />
            {user.name}
          </span>
          <Button variant="outline" size="sm" onClick={() => void signOut()}>
            {t("Sign out")}
          </Button>
        </>
      ) : (
        <Button variant="outline" size="sm" onClick={() => setOpen(true)}>
          {t("Contributor sign-in")}
        </Button>
      )}
      {error && !open && (
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
