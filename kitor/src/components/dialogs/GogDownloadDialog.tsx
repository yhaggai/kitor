import { useEffect } from "react";
import { useGogInstallFlow } from "@/store/gogInstallFlow";

export function GogDownloadDialog({
  open,
  onOpenChange,
  gogId,
  gameId,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  gogId: number;
  gameId: string;
}) {
  useEffect(() => {
    if (open) {
      useGogInstallFlow.getState().show({ gogId, gameId });
      onOpenChange(false);
    }
  }, [open, onOpenChange, gogId, gameId]);
  return null;
}
