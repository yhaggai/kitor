import * as Dialog from "@radix-ui/react-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ExternalLink, Plus, X } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import ReactMarkdown from "react-markdown";
import { Button } from "@/components/ui/button";

type PurchaseGuideDialogProps = {
  scummvmId: string;
  gameTitle: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onAddGame?: () => void;
};

export function PurchaseGuideDialog({
  scummvmId,
  gameTitle,
  open,
  onOpenChange,
  onAddGame,
}: PurchaseGuideDialogProps) {
  const [content, setContent] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadGuide = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const guide = await invoke("get_purchase_guide", { scummvmId });
      setContent(guide as string);
    } catch (err) {
      // If no guide exists, that's okay - we'll show a default message
      setContent("");
      setError(null);
    } finally {
      setLoading(false);
    }
  }, [scummvmId]);

  useEffect(() => {
    if (open) {
      loadGuide();
    }
  }, [open, loadGuide]);

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content className="fixed left-1/2 top-1/2 w-[min(600px,calc(100vw-32px))] max-h-[80vh] -translate-x-1/2 -translate-y-1/2 rounded-lg border bg-background p-6 shadow-lg overflow-y-auto">
          <div className="flex justify-between items-start mb-4">
            <Dialog.Title className="text-lg font-semibold">{gameTitle}</Dialog.Title>
            <Dialog.Close asChild>
              <Button variant="ghost" size="sm" className="h-6 w-6 p-0">
                <X className="h-4 w-4" />
              </Button>
            </Dialog.Close>
          </div>

          {loading && <div className="text-center py-8">Loading guide...</div>}
          {error && <div className="text-center py-8 text-destructive">{error}</div>}
          {!loading && !error && !content && onAddGame && (
            <div className="text-center py-8">
              <p className="text-muted-foreground mb-4">
                This game is not in your library yet. Click the button below to add it.
              </p>
              <Button
                onClick={() => {
                  onAddGame();
                  onOpenChange(false);
                }}
              >
                <Plus className="mr-2 h-4 w-4" />
                Add Game to Library
              </Button>
            </div>
          )}
          {content && (
            <>
              <div className="prose prose-sm dark:prose-invert max-w-none">
                <ReactMarkdown
                  components={{
                    a: ({ href, children }) => (
                      <a
                        href={href}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-1 text-primary hover:underline"
                      >
                        {children}
                        <ExternalLink className="h-3 w-3" />
                      </a>
                    ),
                  }}
                >
                  {content}
                </ReactMarkdown>
              </div>
              {onAddGame && (
                <div className="mt-6 pt-4 border-t flex justify-end gap-2">
                  <Button
                    onClick={() => {
                      onAddGame();
                      onOpenChange(false);
                    }}
                  >
                    <Plus className="mr-2 h-4 w-4" />
                    Add Game to Library
                  </Button>
                </div>
              )}
            </>
          )}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
