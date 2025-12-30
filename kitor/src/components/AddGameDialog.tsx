import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import * as Dialog from "@radix-ui/react-dialog";
import * as Toast from "@radix-ui/react-toast";
import { CheckCircle, FolderSearch, XCircle } from "lucide-react";
import { useCallback, useMemo, useState } from "react";
import { Button } from "@/components/ui/button";

type DetectedGameInfo = {
  scummvm_id: string;
  title: string;
  available_patches_count: number;
};

type AddGameDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAdded: () => Promise<void> | void;
};

type ToastState =
  | { open: false }
  | { open: true; title: string; variant: "default" | "destructive" };

export default function AddGameDialog({ open, onOpenChange, onAdded }: AddGameDialogProps) {
  const [busy, setBusy] = useState(false);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [detected, setDetected] = useState<DetectedGameInfo | null>(null);
  const [toast, setToast] = useState<ToastState>({ open: false });

  const toastClasses = useMemo(() => {
    const base =
      "rounded-md border bg-background px-4 py-3 shadow-lg data-[state=open]:animate-in data-[state=closed]:animate-out";
    if (!toast.open) return base;
    return toast.variant === "destructive"
      ? `${base} border-destructive/50 bg-destructive text-destructive-foreground`
      : base;
  }, [toast]);

  const showToast = useCallback((title: string, variant: ToastState["variant"]) => {
    setToast({ open: true, title, variant });
  }, []);

  const reset = useCallback(() => {
    setBusy(false);
    setSelectedPath(null);
    setDetected(null);
  }, []);

  const handleSelectAndAdd = useCallback(async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Select game installation folder",
      });

      if (!selected) return;

      setBusy(true);
      setSelectedPath(selected);

      const detectedGame = (await invoke("detect_game_in_folder", {
        path: selected,
      })) as DetectedGameInfo | null;

      if (!detectedGame) {
        setDetected(null);
        showToast("No patch match for this folder", "destructive");
        return;
      }

      setDetected(detectedGame);

      await invoke("add_game_manually", {
        scummvmId: detectedGame.scummvm_id,
        title: detectedGame.title,
        installPath: selected,
      });

      showToast(`Added: ${detectedGame.title}`, "default");
      await onAdded();
      onOpenChange(false);
      reset();
    } catch (error) {
      console.error("Failed to add game manually:", error);
      showToast(`Failed to add game: ${error}`, "destructive");
    } finally {
      setBusy(false);
    }
  }, [onAdded, onOpenChange, reset, showToast]);

  return (
    <Toast.Provider swipeDirection="right" duration={3000}>
      <Dialog.Root
        open={open}
        onOpenChange={(nextOpen) => {
          onOpenChange(nextOpen);
          if (!nextOpen) reset();
        }}
      >
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 bg-black/50" />
          <Dialog.Content className="fixed left-1/2 top-1/2 w-[min(520px,calc(100vw-32px))] -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-background p-6 shadow-lg">
            <div className="space-y-1">
              <Dialog.Title className="text-lg font-semibold">Add game</Dialog.Title>
              <Dialog.Description className="text-sm text-muted-foreground">
                Pick a game folder. We’ll auto-detect it and add only if a patch exists.
              </Dialog.Description>
            </div>

            <div className="mt-5 space-y-4">
              <Button onClick={handleSelectAndAdd} disabled={busy} className="w-full">
                <FolderSearch className="mr-2 h-4 w-4" />
                {busy ? "Detecting..." : "Select folder"}
              </Button>

              {selectedPath && (
                <div className="rounded-md border bg-muted/40 p-3 text-sm">
                  <div className="text-muted-foreground">Folder</div>
                  <div className="mt-1 break-all font-mono text-xs">{selectedPath}</div>
                </div>
              )}

              {detected && (
                <div className="flex items-start gap-3 rounded-md border bg-muted/40 p-3">
                  <CheckCircle className="mt-0.5 h-5 w-5 text-green-500" />
                  <div className="min-w-0">
                    <div className="font-medium">{detected.title}</div>
                    <div className="text-sm text-muted-foreground">
                      {detected.scummvm_id} · {detected.available_patches_count} patch(es)
                    </div>
                  </div>
                </div>
              )}

              {!busy && selectedPath && !detected && (
                <div className="flex items-start gap-3 rounded-md border border-destructive/40 bg-muted/40 p-3">
                  <XCircle className="mt-0.5 h-5 w-5 text-destructive" />
                  <div className="min-w-0">
                    <div className="font-medium">No match</div>
                    <div className="text-sm text-muted-foreground">
                      This folder doesn’t match any game with patches in the registry.
                    </div>
                  </div>
                </div>
              )}
            </div>

            <div className="mt-6 flex justify-end gap-2">
              <Dialog.Close asChild>
                <Button variant="outline" disabled={busy}>
                  Close
                </Button>
              </Dialog.Close>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>

      <Toast.Root
        open={toast.open}
        onOpenChange={(nextOpen) => setToast((prev) => (nextOpen ? prev : { open: false }))}
        className={toastClasses}
      >
        {toast.open ? <Toast.Title className="text-sm">{toast.title}</Toast.Title> : null}
      </Toast.Root>
      <Toast.Viewport className="fixed bottom-4 right-4 z-50 flex w-[min(360px,calc(100vw-32px))] flex-col gap-2 outline-none" />
    </Toast.Provider>
  );
}


