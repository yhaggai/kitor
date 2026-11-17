import { useTranslation } from "@kitor/ui";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  AllCommunityModule,
  type CellValueChangedEvent,
  type ColDef,
  ModuleRegistry,
  themeQuartz,
} from "ag-grid-community";
import { AgGridReact } from "ag-grid-react";
import { Redo2, Search, Undo2 } from "lucide-react";
import { type CSSProperties, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import "./TranslationEditor.css";
import type { Contributor } from "@/store/contributor";
import type { ContributionCatalog } from "./TranslationContributionButton";

ModuleRegistry.registerModules([AllCommunityModule]);
type Row = {
  key: string;
  character: string;
  context: string;
  english: string;
  hebrew: string;
  proposal: string;
  variant: string | null;
};
type Draft = {
  base_sha: string;
  base_branch: string;
  rows: Omit<Row, "proposal">[];
  edits: Record<string, string>;
  revision: number;
};
type Preview = {
  digest: string;
  revision: number;
  changes: {
    key: string;
    english: string;
    context: string;
    variant: string | null;
    before: string;
    after: string;
  }[];
};
const theme = themeQuartz.withParams({
  backgroundColor: "#101827",
  foregroundColor: "#e6edf6",
  headerBackgroundColor: "#19243a",
  accentColor: "#efb667",
  borderColor: "#2c3b52",
  fontFamily: "inherit",
  fontSize: 14,
});

export default function TranslationEditor({
  onClose,
  contributor,
  catalog,
}: {
  onClose: () => void;
  contributor: Contributor;
  catalog: ContributionCatalog;
}) {
  const { t, language } = useTranslation();
  const catalogId = catalog.id;
  const drifter = catalogId === "drifter" || catalogId === "drifter-logical";
  const paired = catalog.editable.length === 2;
  const [draft, setDraft] = useState<Draft | null>(null);
  const [data, setData] = useState<Row[]>([]);
  const [edits, setEdits] = useState<Record<string, string>>({});
  const [search, setSearch] = useState("");
  const [changedOnly, setChangedOnly] = useState(false);
  const [message, setMessage] = useState("Loading translation…");
  const [error, setError] = useState("");
  const [closeError, setCloseError] = useState("");
  const [busy, setBusy] = useState(false);
  const [emailOptIn, setEmailOptIn] = useState(false);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [botReady, setBotReady] = useState(false);
  const [botMessage, setBotMessage] = useState("Checking the local contribution bot…");
  const [botError, setBotError] = useState("");
  const [pr, setPr] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const revision = useRef(0),
    latest = useRef<Record<string, string>>({}),
    initialized = useRef(false);
  const queue = useRef<Promise<void>>(Promise.resolve());
  const grid = useRef<AgGridReact<Row>>(null);
  const mounted = useRef(true);
  const generation = useRef(0);
  const checkBot = useCallback(async () => {
    setBotReady(false);
    setBotError("");
    try {
      const status = await invoke<{ configured: boolean; catalogs: string[] }>(
        "contribution_bot_status"
      );
      const supported = catalogId === "drifter" || status.catalogs.includes(catalogId);
      setBotReady(status.configured && supported);
      setBotMessage(
        !supported
          ? "The contribution service does not yet accept this catalog."
          : status.configured
            ? "The bot is ready to open a draft PR."
            : "The local bot is running. Its GitHub App credentials still need to be configured."
      );
    } catch (e) {
      setBotError(String(e));
    }
  }, [catalogId]);
  useEffect(() => {
    mounted.current = true;
    void invoke<Draft>("open_translation_draft", { contributorId: contributor.id, catalogId })
      .then((d) => {
        if (!mounted.current) return;
        revision.current = d.revision;
        latest.current = d.edits;
        setDraft(d);
        setEdits(d.edits);
        setData(d.rows.map((row) => ({ ...row, proposal: d.edits[row.key] ?? row.hebrew })));
        initialized.current = true;
        setSaved(true);
        setMessage("Saved on this computer");
      })
      .catch((e) => setError(String(e)));
    void checkBot();
    return () => {
      mounted.current = false;
    };
  }, [catalogId, checkBot, contributor.id]);

  // Save every committed cell edit through one ordered queue. Stale writes cannot win.
  const persist = useCallback(
    (next: Record<string, string>) => {
      const current = ++generation.current;
      setSaved(false);
      setMessage("Saving draft…");
      queue.current = queue.current
        .catch(() => {})
        .then(async () => {
          const rev = await invoke<number>("save_translation_draft", {
            contributorId: contributor.id,
            catalogId,
            revision: revision.current,
            edits: next,
          });
          revision.current = rev;
          if (mounted.current && current === generation.current) {
            setSaved(true);
            setMessage("Saved on this computer");
            setError("");
            setCloseError("");
          }
        });
      void queue.current.catch((e) => {
        if (mounted.current) {
          setError(String(e));
          setMessage("Draft not saved — keep the editor open");
        }
      });
    },
    [catalogId, contributor.id]
  );
  const onChange = useCallback(
    (event: CellValueChangedEvent<Row>) => {
      const row = event.data;
      if (!row || !initialized.current) return;
      const next = { ...latest.current };
      if (row.proposal === row.hebrew) delete next[row.key];
      else next[row.key] = row.proposal;
      latest.current = next;
      setEdits(next);
      setPreview(null);
      setPr(null);
      persist(next);
      event.api.refreshCells({ rowNodes: [event.node], force: true });
      event.api.onFilterChanged();
    },
    [persist]
  );
  useEffect(() => {
    grid.current?.api.setGridOption("isExternalFilterPresent", () => changedOnly);
    grid.current?.api.onFilterChanged();
  }, [changedOnly]);
  const columns = useMemo<ColDef<Row>[]>(
    () => [
      {
        headerName: t("Speaker"),
        field: "character",
        width: 110,
        hide: catalog.speaker === undefined,
      },
      {
        headerName: t("Edition"),
        field: "variant",
        width: 155,
        hide: !paired,
        valueFormatter: (p) => (p.value ? t(p.value) : ""),
      },
      { headerName: t("Scene / context"), field: "context", width: 200 },
      {
        headerName: t("English"),
        field: "english",
        flex: 1,
        minWidth: 230,
        hide: catalog.english === undefined,
      },
      {
        headerName: t("Current Hebrew"),
        field: "hebrew",
        flex: 1,
        minWidth: 230,
        cellClass: "hebrew-cell",
      },
      {
        headerName: t("Your correction ✎"),
        field: "proposal",
        editable: !busy && !preview,
        flex: 1,
        minWidth: 250,
        cellClass: "hebrew-cell",
        cellEditor: "agLargeTextCellEditor",
        cellEditorPopup: true,
        cellEditorParams: { maxLength: 8000, rows: 5, cols: 50 },
        cellClassRules: { "changed-cell": (p) => p.data?.proposal !== p.data?.hebrew },
      },
    ],
    [busy, catalog.english, catalog.speaker, paired, preview, t]
  );
  const review = async () => {
    grid.current?.api.stopEditing();
    setBusy(true);
    setError("");
    setCloseError("");
    try {
      await queue.current;
      const p = await invoke<Preview>("preview_translation_submission", {
        contributorId: contributor.id,
        catalogId,
      });
      setPreview(p);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  const submit = async () => {
    if (!preview) return;
    setBusy(true);
    setError("");
    setCloseError("");
    try {
      const result = await invoke<{ url: string; existing: boolean; updated?: boolean }>(
        "submit_translation_draft",
        {
          contributorId: contributor.id,
          catalogId,
          revision: preview.revision,
          digest: preview.digest,
          emailOptIn,
        }
      );
      setPr(result.url);
      setMessage(
        result.updated
          ? "Corrections added to your open pull request"
          : result.existing
            ? "This submission already has a pull request"
            : "Draft pull request created"
      );
    } catch (e) {
      setError(String(e));
      void checkBot();
    } finally {
      setBusy(false);
    }
  };
  const close = async () => {
    if (busy) return;
    grid.current?.api.stopEditing();
    try {
      await queue.current;
      onClose();
    } catch (e) {
      setCloseError(String(e));
    }
  };
  const count = Object.keys(edits).length;
  const reviewLabel = (total: number) =>
    t(total === 1 ? "Review {0} correction" : "Review {0} corrections", [total]);
  return (
    <Dialog
      open
      onOpenChange={(value) => {
        if (!value) void close();
      }}
    >
      <DialogContent
        dir={language === "he" ? "rtl" : "ltr"}
        className="translation-editor"
        onInteractOutside={(e) => e.preventDefault()}
      >
        <div className="translation-heading">
          <div>
            <span className="translation-eyebrow">{t("COMMUNITY TRANSLATIONS · EXPERIMENT")}</span>
            <DialogTitle>
              {drifter
                ? t("The Drifter — improve the Hebrew translation")
                : t("{0} — improve the Hebrew translation", [catalog.title])}
            </DialogTitle>
            <DialogDescription>
              {catalog.game === "undertale" && <>{t(catalog.label)} · </>}
              {t("Find a line, edit your correction, then review and send a draft pull request.")}
            </DialogDescription>
          </div>
        </div>
        <p className="translation-notice">
          {t(
            catalogId === "drifter"
              ? "This first POC edits Drifter’s stored text format: Hebrew is in display order. It does not install a patch or publish a release."
              : paired
                ? "Edit Hebrew in normal reading order. If you change the wording, update both the regular and niqqud rows."
                : "Edit Hebrew in normal reading order."
          )}
        </p>
        {(error || closeError) && (
          <p role="alert" className="translation-error">
            {closeError
              ? t("Could not save the draft: {0}. Fix the correction before closing.", [closeError])
              : error}
          </p>
        )}
        {draft && !preview && (
          <>
            <div className="translation-toolbar">
              <label className="translation-search">
                <Search size={16} />
                <input
                  aria-label={t("Search subtitles")}
                  placeholder={t("Search English, Hebrew, speaker or scene…")}
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                />
              </label>
              <label>
                <input
                  type="checkbox"
                  checked={changedOnly}
                  onChange={(e) => setChangedOnly(e.target.checked)}
                />{" "}
                {t("Only my changes ({0})", [count])}
              </label>
              <Button
                variant="outline"
                size="sm"
                onClick={() => grid.current?.api.undoCellEditing()}
                aria-label={t("Undo edit")}
              >
                <Undo2 size={16} />
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => grid.current?.api.redoCellEditing()}
                aria-label={t("Redo edit")}
              >
                <Redo2 size={16} />
              </Button>
            </div>
            <div className="translation-grid" dir="ltr">
              <AgGridReact<Row>
                ref={grid}
                theme={theme}
                rowData={data}
                columnDefs={columns}
                getRowId={(p) => p.data.key}
                defaultColDef={{
                  sortable: true,
                  resizable: true,
                  filter: true,
                  tooltipValueGetter: (p) => String(p.value ?? ""),
                }}
                rowHeight={54}
                quickFilterText={search}
                isExternalFilterPresent={() => changedOnly}
                doesExternalFilterPass={(node) =>
                  !!node.data && node.data.proposal !== node.data.hebrew
                }
                onCellValueChanged={onChange}
                undoRedoCellEditing
                undoRedoCellEditingLimit={100}
                stopEditingWhenCellsLoseFocus
                suppressMovableColumns
              />
            </div>
          </>
        )}
        {preview && !pr && (
          <div className="translation-review">
            <h3>{reviewLabel(preview.changes.length)}</h3>
            <p>
              {t(
                "Your changes will be public on GitHub. Kitor adds them to your open draft pull request for this translation, or opens a new draft if needed."
              )}
            </p>
            {drifter && (
              <>
                <label className="translation-email-opt-in">
                  <input
                    type="checkbox"
                    checked={emailOptIn}
                    disabled={busy}
                    onChange={(event) => setEmailOptIn(event.target.checked)}
                  />
                  {t("Email me when my contribution is approved and included in a patch.")}
                </label>
                <p className="translation-email-disclosure">
                  {t(
                    "Optional. You can contribute without emails. If selected, Kitor stores your verified Google email privately and uses Resend to deliver these updates. Each email includes a link to stop contribution emails. Selecting this again resumes emails for this new contribution."
                  )}
                </p>
              </>
            )}
            {preview.changes.map((change) => (
              <article key={change.key}>
                <p>
                  {change.english || change.context}
                  {change.variant && " · " + t(change.variant)}
                </p>
                <div className="translation-before" dir="auto">
                  − {change.before}
                </div>
                <div className="translation-after" dir="auto">
                  + {change.after}
                </div>
              </article>
            ))}
            {!pr && (
              <div className="translation-auth">
                <h3>{t("Contributing as {0}", [contributor.name])}</h3>
                <p>
                  {t("Public contributor ID: {0}. Your email stays private.", [contributor.id])}
                </p>
                <p>
                  {t(
                    "No GitHub sign-in or repository access is needed from you. The bot posts your reviewed corrections as a public draft PR."
                  )}
                </p>
                <p>{botError || t(botMessage)}</p>
                <Button variant="outline" disabled={busy} onClick={() => void checkBot()}>
                  {t("Check connection")}
                </Button>
              </div>
            )}
          </div>
        )}
        {pr && (
          <section className="translation-celebration" aria-labelledby="contribution-success-title">
            <div className="translation-confetti" aria-hidden="true">
              {Array.from({ length: 48 }, (_, i) => (
                <i
                  // biome-ignore lint/suspicious/noArrayIndexKey: Fixed decorative particles never reorder.
                  key={`confetti-${i}`}
                  style={
                    {
                      "--x": `${(i * 37) % 100}%`,
                      "--delay": `${(i % 8) * 0.09}s`,
                      "--drift": `${((i * 23) % 160) - 80}px`,
                      "--color": ["#efb667", "#83e6bb", "#a69aff", "#83ccff", "#ff9ea9"][i % 5],
                    } as CSSProperties
                  }
                />
              ))}
            </div>
            <div className="translation-celebration-content">
              <span className="translation-celebration-emoji" aria-hidden="true">
                🎉
              </span>
              <span className="translation-eyebrow">{t("A LITTLE EDIT. A BETTER ADVENTURE.")}</span>
              <h3 id="contribution-success-title">{t("Your contribution is in!")}</h3>
              <p className="translation-celebration-thanks">
                {t("Thank you for helping improve the Hebrew translation of {0}.", [catalog.title])}
              </p>
              <p>
                {t(
                  preview?.changes.length === 1
                    ? "Your correction is ready for review. Once approved for release, it can be included in the next patch version."
                    : "Your corrections are ready for review. Once approved for release, they can be included in the next patch version."
                )}
              </p>
              <div className="translation-celebration-next">
                <strong>{t("What happens next?")}</strong>
                <p>
                  {t(
                    "The translation team reviews your changes. You can follow their feedback on your contribution."
                  )}
                </p>
              </div>
              <Button variant="outline" onClick={() => void openUrl(pr)}>
                {t("View your contribution ↗")}
              </Button>
            </div>
          </section>
        )}
        <div className="translation-footer">
          <div aria-live="polite">
            <span className={saved ? "saved-dot" : "saving-dot"} /> {t(message)}
            {draft && (
              <small>
                {t("{0} lines · based on {1}", [
                  draft.rows.length.toLocaleString(language),
                  `${draft.base_branch} / ${draft.base_sha.slice(0, 7)}`,
                ])}
              </small>
            )}
          </div>
          <div className="translation-footer-actions">
            {pr ? (
              <Button onClick={() => void close()}>{t("Done — thank you!")}</Button>
            ) : preview ? (
              <>
                <Button variant="outline" disabled={busy} onClick={() => setPreview(null)}>
                  {t("Back to editing")}
                </Button>
                <Button
                  disabled={busy || !botReady || !preview.changes.length || !!pr}
                  onClick={() => void submit()}
                >
                  {busy ? t("Submitting…") : t("Send corrections")}
                </Button>
              </>
            ) : (
              <Button disabled={!draft || busy || count === 0} onClick={() => void review()}>
                {busy ? t("Preparing…") : reviewLabel(count)}
              </Button>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
