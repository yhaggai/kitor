import * as AlertDialog from "@radix-ui/react-alert-dialog";
import { CheckCircle, Loader2 } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";

import type { PatchDownloadProgress } from "@/hooks/useGameCardState";

type StepStatus = "pending" | "in-progress" | "complete";

type InstallStep = {
  id: string;
  label: string;
  status: StepStatus;
  detail?: string;
};

type InstallPatchDialogProps = {
  updating?: boolean;
  gameTitle: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  installSteps: InstallStep[];
  progress: number;
  download: PatchDownloadProgress | null;
};

function PreparationDetail({ detail }: { detail?: string }) {
  const [seconds, setSeconds] = useState(0);
  useEffect(() => {
    const start = performance.now();
    const timer = window.setInterval(
      () => setSeconds(Math.floor((performance.now() - start) / 1000)),
      1000
    );
    return () => window.clearInterval(timer);
  }, []);

  return (
    <div className="ml-8 w-full space-y-1 text-xs text-muted-foreground">
      <p className="break-words">{detail}</p>
      <p className="tabular-nums">
        {Math.floor(seconds / 60)}:{String(seconds % 60).padStart(2, "0")} elapsed
      </p>
    </div>
  );
}

export function InstallPatchDialog({
  updating = false,
  gameTitle,
  open,
  onOpenChange,
  installSteps,
  progress,
  download,
}: InstallPatchDialogProps) {
  return (
    <AlertDialog.Root open={open} onOpenChange={onOpenChange}>
      <AlertDialog.Portal>
        <AlertDialog.Overlay className="fixed inset-0 bg-black/50" />
        <AlertDialog.Content className="fixed left-1/2 top-1/2 w-[min(500px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-background p-6 shadow-lg">
          <AlertDialog.Title className="text-lg font-semibold mb-4">
            {updating ? "Updating Translation" : "Installing Patch"} for {gameTitle}
          </AlertDialog.Title>
          <div className="mb-5 space-y-2">
            <AlertDialog.Description className="flex items-center justify-between text-sm text-muted-foreground">
              <span>Installation progress</span>
              <span className="font-medium tabular-nums text-foreground">
                {Math.round(progress)}%
              </span>
            </AlertDialog.Description>
            <Progress value={progress} aria-label="Patch installation progress" className="h-2" />
          </div>
          <div className="space-y-3">
            {installSteps.map((step) => (
              <div key={step.id} className="flex flex-wrap items-center gap-3 py-2">
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
                  {step.id === "backup" &&
                  step.status === "in-progress" &&
                  step.detail?.startsWith("Creating backup (")
                    ? "Creating backup"
                    : step.label}
                </span>
                {step.id === "backup" && step.status === "in-progress" && (
                  <PreparationDetail detail={step.detail} />
                )}
                {step.id === "installing" && step.status === "in-progress" && (
                  <p className="ml-8 w-full break-all text-xs text-muted-foreground">
                    {step.detail}
                  </p>
                )}
                {step.id === "downloading" && step.status === "in-progress" && (
                  <div className="ml-8 w-full space-y-2 text-xs text-muted-foreground tabular-nums">
                    <Progress
                      value={download && download.total > 0 ? download.percentage : null}
                      aria-label="Patch download progress"
                      className={`h-2 ${!download || download.total === 0 ? "animate-pulse" : ""}`}
                    />
                    {download ? (
                      <>
                        <div className="flex justify-between gap-2">
                          <span>
                            {(download.downloaded / 1_000_000).toFixed(1)} MB
                            {download.total > 0 &&
                              ` / ${(download.total / 1_000_000).toFixed(1)} MB`}
                          </span>
                          <span>
                            {download.total > 0 ? `${download.percentage}%` : "Size unknown"}
                          </span>
                        </div>
                        <div className="flex justify-between gap-2">
                          <span>
                            {(download.bytes_per_second / 1_000_000).toFixed(2)} MB/s (average)
                          </span>
                          <span>
                            {download.percentage === 100
                              ? "Download complete"
                              : download.seconds_remaining !== null
                                ? `About ${Math.ceil(download.seconds_remaining)}s remaining`
                                : "Estimating time remaining…"}
                          </span>
                        </div>
                      </>
                    ) : (
                      <p>Connecting to download…</p>
                    )}
                  </div>
                )}
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
