import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { StoreInstallButtons } from "@/components/StoreInstallButtons";
import { Button } from "@/components/ui/button";

type PurchaseGuideDialogProps = {
  scummvmId: string;
  gameTitle: string;
  description?: string;
  steamAppId?: number | null;
  gogId?: number | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

const isHebrew = (text: string): boolean => {
  const hebrewPattern = /[\u0590-\u05FF]/;
  return hebrewPattern.test(text);
};

export function PurchaseGuideDialog({
  scummvmId,
  gameTitle,
  description,
  steamAppId,
  gogId,
  open,
  onOpenChange,
}: PurchaseGuideDialogProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content
          aria-describedby={undefined}
          className="fixed left-1/2 top-1/2 w-[min(600px,calc(100vw-32px))] max-h-[80vh] -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-background p-6 shadow-lg overflow-y-auto"
        >
          <div
            className={`flex justify-between items-start mb-4 ${isHebrew(description || gameTitle) ? "flex-row-reverse" : ""}`}
          >
            <Dialog.Title
              className={`text-lg font-semibold text-white ${isHebrew(description || gameTitle) ? "text-right" : ""}`}
              dir={isHebrew(description || gameTitle) ? "rtl" : "ltr"}
            >
              {gameTitle}
            </Dialog.Title>
            <Dialog.Close asChild>
              <Button variant="ghost" size="sm" className="h-6 w-6 p-0" aria-label="Close">
                <X className="h-4 w-4" />
              </Button>
            </Dialog.Close>
          </div>

          {(steamAppId || gogId) && (
            <div className="mb-6 space-y-2">
              <StoreInstallButtons scummvmId={scummvmId} key={scummvmId} steamAppId={steamAppId} gogId={gogId} />
            </div>
          )}

          {description?.trim() && (
            <div className="mb-6 space-y-3 text-sm leading-relaxed text-white">
              {description.split(/\n\s*\n/).map((paragraph) => (
                <p key={paragraph} dir="auto" className="whitespace-pre-line text-start">
                  {paragraph}
                </p>
              ))}
            </div>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
