import { getVersion } from "@tauri-apps/api/app";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Bug, ImagePlus, Mail, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
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
        aria-label={`Remove ${file.name}`}
        onClick={onRemove}
      >
        <X />
      </Button>
    </div>
  );
}

export default function BugReportDialog() {
  const [open, setOpen] = useState(false);
  const [game, setGame] = useState("");
  const [details, setDetails] = useState("");
  const [files, setFiles] = useState<File[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const picker = useRef<HTMLInputElement>(null);

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
          `To: ${BUG_REPORT_EMAIL}\nSubject: ${subject}\n\n${body}`
        );
        setStatus(
          "Report copied. Paste it into an email and attach your screenshots before sending."
        );
      } else if (files.length || encodeURIComponent(body).length > 1500) {
        const draft = await buildEmailDraft(subject, body, files);
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
          "Email file saved with any selected screenshots. Open it in your email app and send it. If it opens read-only, forward it to the address above. For webmail, attach the saved .eml file to a new email."
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
    "w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline">
          <Bug />
          Report a bug
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[90vh] max-w-xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Report a bug</DialogTitle>
          <DialogDescription>
            Help us fix Kitor. Your report goes to{" "}
            <span className="select-all break-all">{BUG_REPORT_EMAIL}</span>.
          </DialogDescription>
        </DialogHeader>
        <div className="rounded-md bg-muted p-3 text-sm">
          <p className="font-medium">What makes a useful report?</p>
          <ul className="mt-1 list-disc space-y-1 pl-5 text-muted-foreground">
            <li>Name the game and patch, and where you got the game (Steam, GOG, etc.).</li>
            <li>List the steps to reproduce, what you expected, and what actually happened.</li>
            <li>Include the exact error and a screenshot. Crop out personal information.</li>
          </ul>
        </div>
        <form
          className="space-y-4"
          onSubmit={(event) => {
            event.preventDefault();
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
            <label htmlFor="bug-game" className="text-sm font-medium">
              Game / patch (optional)
            </label>
            <input
              id="bug-game"
              className={fieldClass}
              value={game}
              maxLength={200}
              onChange={(event) => setGame(event.target.value)}
              placeholder="e.g. Space Quest V, Hebrew patch, GOG"
            />
          </div>
          <div className="space-y-1">
            <label htmlFor="bug-details" className="text-sm font-medium">
              What went wrong?
            </label>
            <textarea
              id="bug-details"
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
              Add screenshots
            </Button>
            <input
              ref={picker}
              type="file"
              className="sr-only"
              tabIndex={-1}
              aria-label="Choose screenshots"
              accept={SCREENSHOT_TYPES.join(",")}
              multiple
              onChange={(event) => {
                addFiles(Array.from(event.target.files ?? []));
                event.target.value = "";
              }}
            />
            <p className="text-xs text-muted-foreground">
              Choose files or paste a screenshot into this form. Up to 5 images, 10 MB total (PNG,
              JPEG, WebP).
            </p>
            <p className="text-xs text-muted-foreground">
              Capture an area: Windows — Win + Shift + S; Mac — Control + Shift + Command + 4. Then
              paste here.
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
          <p className="text-xs text-muted-foreground">
            Kitor version and operating system are included. You send the email yourself. With
            screenshots or a long report, we save an email file (.eml) with all attachments for you
            to open or attach in your email app.
          </p>
          {error && (
            <p role="alert" className="text-sm text-destructive">
              {error}
            </p>
          )}
          {status && <output className="block text-sm">{status}</output>}
          <div className="flex flex-wrap justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              disabled={busy || !details.trim()}
              onClick={() => void prepareReport(true)}
            >
              Copy report
            </Button>
            <Button type="submit" disabled={busy || !details.trim()}>
              <Mail />
              {busy ? "Preparing…" : files.length ? "Save email with screenshots" : "Prepare email"}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
