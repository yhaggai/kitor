import * as AlertDialog from "@radix-ui/react-alert-dialog";
import { Undo } from "lucide-react";
import { Button } from "@/components/ui/button";

type RestoreGameDialogProps = {
  gameTitle: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
  isRestoring: boolean;
};

export function RestoreGameDialog({
  gameTitle,
  open,
  onOpenChange,
  onConfirm,
  isRestoring,
}: RestoreGameDialogProps) {
  return (
    <AlertDialog.Root open={open} onOpenChange={onOpenChange}>
      <AlertDialog.Portal>
        <AlertDialog.Overlay className="fixed inset-0 bg-black/50" />
        <AlertDialog.Content className="fixed left-1/2 top-1/2 w-[min(420px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-background p-6 shadow-lg">
          <AlertDialog.Title className="text-lg font-semibold">
            Restore {gameTitle}?
          </AlertDialog.Title>
          <AlertDialog.Description className="mt-2 text-sm text-muted-foreground">
            This will restore the game to its original state before the patch was installed. The
            patch will be removed and the backup files will be restored.
          </AlertDialog.Description>
          <div className="mt-6 flex justify-end gap-2">
            <AlertDialog.Cancel asChild>
              <Button variant="outline" disabled={isRestoring} className="text-foreground">
                Cancel
              </Button>
            </AlertDialog.Cancel>
            <AlertDialog.Action asChild>
              <Button onClick={onConfirm} disabled={isRestoring}>
                <Undo className="mr-2 h-4 w-4" />
                {isRestoring ? "Restoring..." : "Restore"}
              </Button>
            </AlertDialog.Action>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
