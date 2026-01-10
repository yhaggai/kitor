import * as AlertDialog from "@radix-ui/react-alert-dialog";
import { CheckCircle, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";

type StepStatus = "pending" | "in-progress" | "complete";

type InstallStep = {
  id: string;
  label: string;
  status: StepStatus;
};

type InstallPatchDialogProps = {
  gameTitle: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  installSteps: InstallStep[];
};

export function InstallPatchDialog({
  gameTitle,
  open,
  onOpenChange,
  installSteps,
}: InstallPatchDialogProps) {
  return (
    <AlertDialog.Root open={open} onOpenChange={onOpenChange}>
      <AlertDialog.Portal>
        <AlertDialog.Overlay className="fixed inset-0 bg-black/50" />
        <AlertDialog.Content className="fixed left-1/2 top-1/2 w-[min(500px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-background p-6 shadow-lg">
          <AlertDialog.Title className="text-lg font-semibold mb-4">
            Installing Patch for {gameTitle}
          </AlertDialog.Title>
          <div className="space-y-3">
            {installSteps.map((step) => (
              <div key={step.id} className="flex items-center gap-3 py-2">
                <div className="flex-shrink-0 w-5 h-5">
                  {step.status === "complete" && <CheckCircle className="w-5 h-5 text-green-600" />}
                  {step.status === "in-progress" && (
                    <Loader2 className="w-5 h-5 text-primary animate-spin" />
                  )}
                  {step.status === "pending" && <div className="w-5 h-5" />}
                </div>
                <span
                  className={
                    step.status === "pending"
                      ? "text-muted-foreground"
                      : step.status === "complete"
                        ? "text-foreground"
                        : "text-foreground font-medium"
                  }
                >
                  {step.label}
                </span>
              </div>
            ))}
          </div>
          <div className="mt-6 flex justify-end">
            <AlertDialog.Cancel asChild>
              <Button variant="outline" className="text-foreground">
                Close
              </Button>
            </AlertDialog.Cancel>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
