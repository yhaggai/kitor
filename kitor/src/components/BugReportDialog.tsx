import { type AnimatedIconHandle, BugIcon, useTranslation } from "@kitor/ui";
import { getVersion } from "@tauri-apps/api/app";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ImagePlus, Mail, X } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  BUG_REPORT_EMAIL,
  buildEmailDraft,
  MAX_SCREENSHOT_BYTES,
  MAX_SCREENSHOTS,
  reportBody,
  SCREENSHOT_TYPES,
} from "@/lib/bugReport";

function Screenshot({ file, onRemove }: { file: File; onRemove: () => void }) {
  const { t } = useTranslation();
  const [url, setUrl] = useState("");
  useEffect(() => {
    const preview = URL.createObjectURL(file);
    setUrl(preview);
    return () => URL.revokeObjectURL(preview);
  }, [file]);
  return (
    <div className="relative rounded-md border p-2">
      <img src={url} alt={file.name} className="h-20 w-full object-contain" />
      <p className="mt-1 truncate text-xs" title={file.name}>
        {file.name}
      </p>
      <Button
        type="button"
        size="icon"
        variant="secondary"
        className="absolute right-1 top-1 h-6 w-6"
        aria-label={t("Remove {0}", [file.name])}
        onClick={onRemove}
      >
        <X />
      </Button>
    </div>
  );
}

export default function BugReportDialog({
  game: initialGame = "",
  details: initialDetails = "",
}: {
  game?: string;
  details?: string;
} = {}) {
  const id = useId();
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [game, setGame] = useState(initialGame);
  const [details, setDetails] = useState(initialDetails);
  const [files, setFiles] = useState<File[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [diagnostics, setDiagnostics] = useState<string>();
  const [collecting, setCollecting] = useState(false);
  const picker = useRef<HTMLInputElement>(null);
  const iconRef = useRef<AnimatedIconHandle>(null);

  // Failure entry points supply details; the general report button stays opt-in.
  useEffect(() => {
    if (!open || !initialDetails || !isTauri()) return;
    let active = true;
    setDiagnostics(undefined);
    setCollecting(true);
    void invoke<string>("collect_bug_diagnostics")
      .then((result) => {
        if (active) setDiagnostics(result);
      })
      .catch(() => {
        if (active)
          setError("Could not collect diagnostics. You can still send a report without them.");
      })
      .finally(() => {
        if (active) setCollecting(false);
      });
    return () => {
      active = false;
    };
  }, [open, initialDetails]);

  function addFiles(incoming: File[]) {
    setError("");
    setStatus("");
    if (incoming.some((file) => !SCREENSHOT_TYPES.includes(file.type))) {
      setError("Please choose PNG, JPEG, or WebP screenshots.");
      return;
    }
    const next = [...files, ...incoming];
    if (
      next.length > MAX_SCREENSHOTS ||
      next.reduce((sum, file) => sum + file.size, 0) > MAX_SCREENSHOT_BYTES
    ) {
      setError("Add up to 5 screenshots, totaling no more than 10 MB.");
      return;
    }
    setFiles(next);
  }

  async function prepareReport(copyOnly = false) {
    setBusy(true);
    setError("");
    setStatus("");
    try {
      const [version, os] = isTauri()
        ? await Promise.all([getVersion(), invoke<string>("get_os")])
        : ["Browser preview", navigator.platform];
      const body = reportBody(game, details, version, os);
      const subject = "Kitor bug report";
      if (copyOnly) {
        await navigator.clipboard.writeText(
          `To: ${BUG_REPORT_EMAIL}\nSubject: ${subject}\n\n${body}${diagnostics ? `\n\nDiagnostics:\n${diagnostics}` : ""}`
        );
        setStatus(
          "Report copied. Paste it into an email and attach your screenshots before sending."
        );
      } else if (diagnostics || files.length || encodeURIComponent(body).length > 1500) {
        const draft = await buildEmailDraft(subject, body, files, diagnostics);
        if (isTauri()) {
          const saved = await invoke<boolean>("save_bug_report", { draft });
          if (!saved) return;
        } else {
          const url = URL.createObjectURL(new Blob([draft], { type: "message/rfc822" }));
          const link = document.createElement("a");
          link.href = url;
          link.download = "kitor-bug-report.eml";
          link.click();
          setTimeout(() => URL.revokeObjectURL(url), 1000);
        }
        setStatus(
          "Email file saved with the selected attachments. Open it in your email app and send it. If it opens read-only, forward it to the address above. For webmail, attach the saved .eml file to a new email."
        );
      } else {
        const url = `mailto:${BUG_REPORT_EMAIL}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
        if (isTauri()) await openUrl(url);
        else window.location.href = url;
        setStatus(
          "Email app requested. Review the report and press Send there. If nothing opens, use Copy report and paste it into your webmail."
        );
      }
    } catch {
      setError(
        "Could not prepare the email. Try again, or copy the report and email it to the address above."
      );
    } finally {
      setBusy(false);
    }
  }

  const fieldClass =
    "kt-request-field w-full rounded-md border border-input bg-background px-3 py-2 text-sm";
  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (nextOpen && initialDetails) {
          setGame(initialGame);
          setDetails(initialDetails);
          setStatus("");
          setError("");
        }
        if (nextOpen && initialDetails && isTauri()) setCollecting(true);
        setOpen(nextOpen);
      }}
    >
      <DialogTrigger asChild>
        <Button
          type="button"
          variant="outline"
          onMouseEnter={() => iconRef.current?.startAnimation()}
          onMouseLeave={() => iconRef.current?.stopAnimation()}
        >
          <BugIcon ref={iconRef} size={16} />
          <span>{t("Report a bug")}</span>
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[90vh] max-w-xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t("Report a bug")}</DialogTitle>
          <DialogDescription>
            {t("Help us fix Kitor. Your report goes to")}{" "}
            <span className="select-all break-all">{BUG_REPORT_EMAIL}</span>.
          </DialogDescription>
        </DialogHeader>
        <div className="rounded-md bg-muted p-3 text-sm">
          <p className="font-medium">{t("What makes a useful report?")}</p>
          <ul className="mt-1 list-disc space-y-1 pl-5 text-muted-foreground">
            <li>{t("Name the game and patch, and where you got the game (Steam, GOG, etc.).")}</li>
            <li>
              {t("List the steps to reproduce, what you expected, and what actually happened.")}
            </li>
            <li>{t("Include the exact error and a screenshot. Crop out personal information.")}</li>
          </ul>
        </div>
        <form
          className="space-y-4"
          onSubmit={(event) => {
            event.preventDefault();
            event.stopPropagation();
            void prepareReport();
          }}
          onPaste={(event) => {
            const images = Array.from(event.clipboardData.files);
            if (images.length) {
              event.preventDefault();
              addFiles(images);
            }
          }}
        >
          <div className="space-y-1">
            <label htmlFor={`${id}-game`} className="text-sm font-medium">
              {t("Game / patch (optional)")}{" "}
            </label>
            <input
              id={`${id}-game`}
              className={fieldClass}
              value={game}
              maxLength={200}
              onChange={(event) => setGame(event.target.value)}
              placeholder={t("e.g. Space Quest V, Hebrew patch, GOG")}
            />
          </div>
          <div className="space-y-1">
            <label htmlFor={`${id}-details`} className="text-sm font-medium">
              {t("What went wrong?")}{" "}
            </label>
            <textarea
              id={`${id}-details`}
              className={`${fieldClass} min-h-32`}
              required
              maxLength={12000}
              value={details}
              onChange={(event) => setDetails(event.target.value)}
              placeholder={"1. I tried to…\n2. I expected…\n3. Instead…\nError message:"}
            />
          </div>
          <div className="space-y-2 rounded-md border border-dashed p-3">
            <Button
              type="button"
              variant="outline"
              onClick={() => picker.current?.click()}
              disabled={busy || files.length >= MAX_SCREENSHOTS}
            >
              <ImagePlus />
              {t("Add screenshots")}{" "}
            </Button>
            <input
              ref={picker}
              type="file"
              className="sr-only"
              tabIndex={-1}
              aria-label={t("Choose screenshots")}
              accept={SCREENSHOT_TYPES.join(",")}
              multiple
              onChange={(event) => {
                addFiles(Array.from(event.target.files ?? []));
                event.target.value = "";
              }}
            />
            <p className="text-xs text-muted-foreground">
              {t(
                "Choose files or paste a screenshot into this form. Up to 5 images, 10 MB total (PNG, JPEG, WebP)."
              )}{" "}
            </p>
            <p className="text-xs text-muted-foreground">
              {t(
                "Capture an area: Windows — Win + Shift + S; Mac — Control + Shift + Command + 4. Then paste here."
              )}{" "}
            </p>
            {files.length > 0 && (
              <div className="grid grid-cols-3 gap-2">
                {files.map((file, index) => (
                  <Screenshot
                    key={`${file.name}-${file.lastModified}-${index}`}
                    file={file}
                    onRemove={() => setFiles(files.filter((_, i) => i !== index))}
                  />
                ))}
              </div>
            )}
          </div>
          {isTauri() && (
            <div className="space-y-2 rounded-md border p-3">
              <label className="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={collecting || !!diagnostics}
                  disabled={busy || collecting}
                  onChange={async (event) => {
                    if (!event.target.checked) {
                      setDiagnostics(undefined);
                      return;
                    }
                    setCollecting(true);
                    setError("");
                    try {
                      setDiagnostics(await invoke<string>("collect_bug_diagnostics"));
                    } catch {
                      setError(
                        "Could not collect diagnostics. You can still send a report without them."
                      );
                    } finally {
                      setCollecting(false);
                    }
                  }}
                />
                {collecting ? t("Collecting diagnostics…") : t("Include diagnostics")}
              </label>
              <p className="text-xs text-muted-foreground">
                {t(
                  "Includes game IDs, patch status, runtime information and recent launch/patch attempts for your library. Excludes personal paths, credentials, saves and game contents."
                )}{" "}
              </p>
              {diagnostics && (
                <details>
                  <summary className="cursor-pointer text-sm">{t("Preview diagnostics")}</summary>
                  <pre
                    className="max-h-48 overflow-auto whitespace-pre-wrap break-all text-xs"
                    dir="ltr"
                  >
                    {diagnostics}
                  </pre>
                </details>
              )}
            </div>
          )}
          <p className="text-xs text-muted-foreground">
            {t(
              "Kitor version and operating system are included. You send the email yourself. With diagnostics, screenshots or a long report, we save an email file (.eml) with all attachments for you to open or attach in your email app."
            )}{" "}
          </p>
          {error && (
            <p role="alert" className="text-sm text-destructive">
              {t(error)}
            </p>
          )}
          {status && <output className="block text-sm">{t(status)}</output>}
          <div className="flex flex-wrap justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              disabled={busy || collecting || !details.trim()}
              onClick={() => void prepareReport(true)}
            >
              {t("Copy report")}{" "}
            </Button>
            <Button type="submit" disabled={busy || collecting || !details.trim()}>
              <Mail />
              {busy
                ? t("Preparing…")
                : files.length || diagnostics
                  ? t("Save email with attachments")
                  : t("Prepare email")}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
