import { type ReactNode, useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "../i18n";
import { type AnimatedIconHandle, SettingsIcon } from "../icons";

export function ManageMenu({
  children,
  label = "Manage game",
}: {
  children: ReactNode;
  label?: string;
}) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLFieldSetElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const iconRef = useRef<AnimatedIconHandle>(null);
  const id = useId();
  useEffect(() => {
    if (!open) return;
    const close = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", close);
    root.current
      ?.querySelector<HTMLButtonElement>(".kt-manage-dropdown button:not(:disabled)")
      ?.focus();
    return () => document.removeEventListener("pointerdown", close);
  }, [open]);
  return (
    <fieldset
      aria-label={t(label)}
      className="kt-manage-menu"
      ref={root}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) setOpen(false);
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          setOpen(false);
          trigger.current?.focus();
        }
        if (open && ["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
          event.preventDefault();
          const buttons = Array.from(
            root.current?.querySelectorAll<HTMLButtonElement>(
              ".kt-manage-dropdown button:not(:disabled)"
            ) ?? []
          );
          const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
          const next =
            event.key === "Home"
              ? 0
              : event.key === "End"
                ? buttons.length - 1
                : (current + (event.key === "ArrowDown" ? 1 : -1) + buttons.length) %
                  buttons.length;
          buttons[next]?.focus();
        }
      }}
    >
      <button
        ref={trigger}
        type="button"
        className="kt-manage-trigger"
        aria-label={t(label)}
        title={t(label)}
        aria-expanded={open}
        aria-controls={id}
        onClick={() => setOpen((value) => !value)}
        onMouseEnter={() => iconRef.current?.startAnimation()}
        onMouseLeave={() => iconRef.current?.stopAnimation()}
      >
        <SettingsIcon ref={iconRef} size={22} />
      </button>
      {open && (
        // biome-ignore lint/a11y/useKeyWithClickEvents: Delegates clicks from native buttons, including keyboard activation.
        <fieldset
          aria-label={t(label)}
          id={id}
          className="kt-manage-dropdown"
          onClick={(event) => {
            if ((event.target as HTMLElement).closest("button")) {
              setOpen(false);
              trigger.current?.focus();
            }
          }}
        >
          {children}
        </fieldset>
      )}
    </fieldset>
  );
}
