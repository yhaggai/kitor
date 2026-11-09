import { PencilLine } from "lucide-react";
import { lazy, Suspense, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { useContributor } from "@/store/contributor";
import { GoogleSignIn } from "./ContributorAccount";

const Editor = lazy(() => import("./TranslationEditor"));
export function TranslationContributionButton({ gameId }: { gameId: string }) {
  const [open, setOpen] = useState(false);
  const user = useContributor((s) => s.user);
  if (gameId !== "drifter") return null;
  return (
    <>
      <Button variant="outline" className="w-full" onClick={() => setOpen(true)}>
        <PencilLine className="mr-2 h-4 w-4" />
        Improve translation · POC
      </Button>
      {open && !user && (
        <Dialog open onOpenChange={setOpen}>
          <DialogContent>
            <DialogTitle>Sign in to improve this translation</DialogTitle>
            <DialogDescription>
              Continue with your Google account, then edit and review your corrections.
            </DialogDescription>
            <GoogleSignIn />
          </DialogContent>
        </Dialog>
      )}
      {open && user && (
        <Suspense fallback={<output>Opening translation editor…</output>}>
          <Editor key={user.id} contributor={user} onClose={() => setOpen(false)} />
        </Suspense>
      )}
    </>
  );
}
