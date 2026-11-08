import { PencilLine } from "lucide-react";
import { lazy, Suspense, useState } from "react";
import { Button } from "@/components/ui/button";

const Editor = lazy(() => import("./TranslationEditor"));
export function TranslationContributionButton({ gameId }: { gameId: string }) {
  const [open, setOpen] = useState(false);
  if (gameId !== "drifter") return null;
  return (
    <>
      <Button variant="outline" className="w-full" onClick={() => setOpen(true)}>
        <PencilLine className="mr-2 h-4 w-4" />
        Improve translation · POC
      </Button>
      {open && (
        <Suspense fallback={<output>Opening translation editor…</output>}>
          <Editor onClose={() => setOpen(false)} />
        </Suspense>
      )}
    </>
  );
}
