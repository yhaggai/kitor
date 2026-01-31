import { createContext, type ReactNode, useCallback, useState } from "react";
import { createPortal } from "react-dom";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

type AlertOptions = {
  title: string;
  description?: string;
};

type ConfirmOptions = {
  title: string;
  description?: string;
  confirmText?: string;
  cancelText?: string;
};

type CustomDialogOptions = {
  content: ReactNode;
  onClose?: () => void;
};

type DialogContextType = {
  alert: (options: AlertOptions) => Promise<void>;
  confirm: (options: ConfirmOptions) => Promise<boolean>;
  showDialog: (options: CustomDialogOptions) => void;
  hideDialog: () => void;
};

export const DialogContext = createContext<DialogContextType | null>(null);

type DialogState =
  | { type: "closed" }
  | { type: "alert"; options: AlertOptions; resolve: () => void }
  | { type: "confirm"; options: ConfirmOptions; resolve: (value: boolean) => void }
  | { type: "custom"; options: CustomDialogOptions };

type DialogProviderProps = {
  children: ReactNode;
  dialogRootElement: HTMLElement;
};

export function DialogProvider({ children, dialogRootElement }: DialogProviderProps) {
  const [state, setState] = useState<DialogState>({ type: "closed" });

  const alert = useCallback((options: AlertOptions): Promise<void> => {
    return new Promise((resolve) => {
      setState({ type: "alert", options, resolve });
    });
  }, []);

  const confirm = useCallback((options: ConfirmOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      setState({ type: "confirm", options, resolve });
    });
  }, []);

  const showDialog = useCallback((options: CustomDialogOptions) => {
    setState({ type: "custom", options });
  }, []);

  const hideDialog = useCallback(() => {
    setState({ type: "closed" });
  }, []);

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      if (state.type === "alert") {
        state.resolve();
      } else if (state.type === "confirm") {
        state.resolve(false);
      } else if (state.type === "custom") {
        state.options.onClose?.();
      }
      setState({ type: "closed" });
    }
  };

  const handleConfirm = () => {
    if (state.type === "confirm") {
      state.resolve(true);
      setState({ type: "closed" });
    }
  };

  const handleAlertOk = () => {
    if (state.type === "alert") {
      state.resolve();
      setState({ type: "closed" });
    }
  };

  const dialogContent = (() => {
    if (state.type === "closed") return null;

    if (state.type === "alert") {
      return (
        <Dialog open onOpenChange={handleOpenChange}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{state.options.title}</DialogTitle>
              {state.options.description && (
                <DialogDescription>{state.options.description}</DialogDescription>
              )}
            </DialogHeader>
            <DialogFooter>
              <Button onClick={handleAlertOk}>OK</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      );
    }

    if (state.type === "confirm") {
      return (
        <Dialog open onOpenChange={handleOpenChange}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{state.options.title}</DialogTitle>
              {state.options.description && (
                <DialogDescription>{state.options.description}</DialogDescription>
              )}
            </DialogHeader>
            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => handleOpenChange(false)}
                className="text-white"
              >
                {state.options.cancelText || "Cancel"}
              </Button>
              <Button onClick={handleConfirm}>{state.options.confirmText || "Confirm"}</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      );
    }

    if (state.type === "custom") {
      return (
        <Dialog open onOpenChange={handleOpenChange}>
          <DialogContent>{state.options.content}</DialogContent>
        </Dialog>
      );
    }

    return null;
  })();

  return (
    <DialogContext.Provider value={{ alert, confirm, showDialog, hideDialog }}>
      {children}
      {dialogContent && createPortal(dialogContent, dialogRootElement)}
    </DialogContext.Provider>
  );
}
