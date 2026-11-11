import { useTranslation } from "@kitor/ui";
import { isTauri } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Languages, Mail } from "lucide-react";
import { useState } from "react";
import BugReportDialog from "@/components/BugReportDialog";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { BUG_REPORT_EMAIL } from "@/lib/bugReport";

export default function TranslationRequestDialog() {
  const { t } = useTranslation();
  const [game, setGame] = useState("");
  const [storeUrl, setStoreUrl] = useState("");
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");

  async function prepareRequest(copyOnly = false) {
    if (!game.trim() || busy) return;
    setBusy(true);
    setError("");
    setStatus("");
    const subject = "Kitor Hebrew translation request";
    const body = `I'd like to request a Hebrew translation.\n\nGame: ${game.trim()}\nStore link: ${storeUrl.trim() || t("Not provided")}\n\n${notes.trim()}`;
    try {
      if (copyOnly) {
        await navigator.clipboard.writeText(
          `To: ${BUG_REPORT_EMAIL}\nSubject: ${subject}\n\n${body}`
        );
        setStatus("Request copied. Paste it into your email app and send it.");
      } else {
        const url = `mailto:${BUG_REPORT_EMAIL}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
        if (isTauri()) await openUrl(url);
        else window.location.href = url;
        setStatus(
          "Email app requested. Review your request and press Send there. If nothing opens, use Copy request for webmail."
        );
      }
    } catch {
      setError(
        "Could not prepare the request. Try again, or copy the form details into an email to the address above."
      );
    } finally {
      setBusy(false);
    }
  }

  const fieldClass =
    "kt-request-field w-full rounded-md border border-input bg-background px-3 py-2 text-sm";

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline">
          <Languages />
          <span>{t("Request a translation")}</span>
        </Button>
      </DialogTrigger>
      <DialogContent className="max-h-[90vh] max-w-xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{t("Request a translation")}</DialogTitle>
          <DialogDescription>
            {t("Which game would you like to play in Hebrew? Send your suggestion to")}{" "}
            <span className="select-all break-all">{BUG_REPORT_EMAIL}</span>.
          </DialogDescription>
        </DialogHeader>
        <form
          className="space-y-4"
          onSubmit={(event) => {
            event.preventDefault();
            void prepareRequest();
          }}
          onChange={() => {
            setStatus("");
            setError("");
          }}
        >
          <div className="space-y-1">
            <label htmlFor="translation-game" className="text-sm font-medium">
              {t("Game name")}{" "}
            </label>
            <input
              id="translation-game"
              className={fieldClass}
              required
              maxLength={150}
              value={game}
              onChange={(event) => setGame(event.target.value)}
              placeholder={t("Name of the game and edition")}
            />
          </div>
          <div className="space-y-1">
            <label htmlFor="translation-store" className="text-sm font-medium">
              {t("Store link (optional)")}{" "}
            </label>
            <input
              id="translation-store"
              type="url"
              className={fieldClass}
              maxLength={300}
              value={storeUrl}
              onChange={(event) => setStoreUrl(event.target.value)}
              placeholder="https://store.steampowered.com/…"
            />
          </div>
          <div className="space-y-1">
            <label htmlFor="translation-notes" className="text-sm font-medium">
              {t("Additional details (optional)")}{" "}
            </label>
            <textarea
              id="translation-notes"
              className={`${fieldClass} min-h-28`}
              maxLength={500}
              value={notes}
              onChange={(event) => setNotes(event.target.value)}
              placeholder={t(
                "Tell us why you'd love a translation, or share an existing translation you know about."
              )}
            />
          </div>
          <p className="text-xs text-muted-foreground">
            {t(
              "You send the email yourself. Requests help us understand interest; a translation is not guaranteed."
            )}{" "}
          </p>
          {error && (
            <div className="space-y-2">
              <BugReportDialog details={t(error)} />
              <p role="alert" className="text-sm text-destructive">
                {t(error)}
              </p>
            </div>
          )}
          {status && <output className="block text-sm">{t(status)}</output>}
          <div className="flex flex-wrap justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              disabled={busy || !game.trim()}
              onClick={() => void prepareRequest(true)}
            >
              {t("Copy request")}{" "}
            </Button>
            <Button type="submit" disabled={busy || !game.trim()}>
              <Mail />
              {busy ? t("Preparing…") : t("Prepare email")}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
