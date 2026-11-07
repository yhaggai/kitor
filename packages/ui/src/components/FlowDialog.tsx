import { type ReactNode, useEffect, useId, useRef } from "react";
import { useTranslation } from "../i18n";
import { Button } from "./Button";

/** Native modal gives every flow focus containment, Escape and focus restoration. */
export function FlowDialog({
  title,
  description,
  children,
  footer,
  onClose,
  dismissible = true,
  className = "",
}: {
  title: string;
  description?: string;
  children?: ReactNode;
  footer?: ReactNode;
  onClose: () => void;
  dismissible?: boolean;
  className?: string;
}) {
  const { t } = useTranslation();
  const ref = useRef<HTMLDialogElement>(null);
  const id = useId();
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    ref.current?.showModal();
    return () => {
      ref.current?.close();
      previous?.focus();
    };
  }, []);
  return (
    <dialog
      ref={ref}
      className={`kitor-theme kt-dialog ${className}`}
      aria-labelledby={id}
      aria-describedby={description ? `${id}-description` : undefined}
      onCancel={(event) => {
        event.preventDefault();
        if (dismissible) onClose();
      }}
    >
      <header>
        <p className="kt-eyebrow">KITOR</p>
        <h2 id={id}>{title}</h2>
        {description && <p id={`${id}-description`}>{description}</p>}
      </header>
      <div className="kt-flow-body">{children}</div>
      <footer className="kt-flow-actions">
        {dismissible && (
          <Button variant="secondary" onClick={onClose}>
            {t("Close")}{" "}
          </Button>
        )}
        {footer}
      </footer>
    </dialog>
  );
}

export function OperationProgress({ value, step }: { value: number; step: string }) {
  return (
    <div className="kt-progress">
      <label>
        {step}
        <progress aria-label={step} value={value} max={100} />
      </label>
      <output aria-live="polite">
        {value}% · {step}
      </output>
    </div>
  );
}

export function ActionCard({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <section className="kt-action-card">
      <h2>{title}</h2>
      {description && <p>{description}</p>}
      <div className="kt-flow-body">{children}</div>
    </section>
  );
}
