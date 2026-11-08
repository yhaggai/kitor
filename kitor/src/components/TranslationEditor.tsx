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
import { Github, Redo2, Search, Undo2 } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import "./TranslationEditor.css";

ModuleRegistry.registerModules([AllCommunityModule]);
type Row = {
  key: string;
  character: string;
  context: string;
  english: string;
  hebrew: string;
  proposal: string;
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
  changes: { key: string; english: string; before: string; after: string }[];
};
type Auth = { connected: boolean; configured: boolean };
type Login = { user_code: string; verification_uri: string; interval: number };
const theme = themeQuartz.withParams({
  backgroundColor: "#101827",
  foregroundColor: "#e6edf6",
  headerBackgroundColor: "#19243a",
  accentColor: "#efb667",
  borderColor: "#2c3b52",
  fontFamily: "inherit",
  fontSize: 14,
});

export default function TranslationEditor({ onClose }: { onClose: () => void }) {
  const [draft, setDraft] = useState<Draft | null>(null);
  const [data, setData] = useState<Row[]>([]);
  const [edits, setEdits] = useState<Record<string, string>>({});
  const [search, setSearch] = useState("");
  const [changedOnly, setChangedOnly] = useState(false);
  const [message, setMessage] = useState("Loading translation…");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [auth, setAuth] = useState<Auth>({ connected: false, configured: false });
  const [login, setLogin] = useState<Login | null>(null);
  const [pr, setPr] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const revision = useRef(0),
    latest = useRef<Record<string, string>>({}),
    initialized = useRef(false);
  const queue = useRef<Promise<void>>(Promise.resolve());
  const grid = useRef<AgGridReact<Row>>(null);
  const mounted = useRef(true);
  const generation = useRef(0);
  const authReady = useCallback(
    () =>
      invoke<Auth>("contribution_auth_status")
        .then(setAuth)
        .catch((e) => setError(String(e))),
    []
  );
  useEffect(() => {
    mounted.current = true;
    void invoke<Draft>("open_translation_draft")
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
    void authReady();
    return () => {
      mounted.current = false;
    };
  }, [authReady]);

  // Save every committed cell edit through one ordered queue. Stale writes cannot win.
  const persist = useCallback((next: Record<string, string>) => {
    const current = ++generation.current;
    setSaved(false);
    setMessage("Saving draft…");
    queue.current = queue.current
      .catch(() => {})
      .then(async () => {
        const rev = await invoke<number>("save_translation_draft", {
          revision: revision.current,
          edits: next,
        });
        revision.current = rev;
        if (mounted.current && current === generation.current) {
          setSaved(true);
          setMessage("Saved on this computer");
          setError("");
        }
      });
    void queue.current.catch((e) => {
      if (mounted.current) {
        setError(String(e));
        setMessage("Draft not saved — keep the editor open");
      }
    });
  }, []);
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
      { headerName: "Speaker", field: "character", width: 110 },
      { headerName: "Scene / context", field: "context", width: 200 },
      { headerName: "English", field: "english", flex: 1, minWidth: 230 },
      {
        headerName: "Current Hebrew",
        field: "hebrew",
        flex: 1,
        minWidth: 230,
        cellClass: "hebrew-cell",
      },
      {
        headerName: "Your correction ✎",
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
    [busy, preview]
  );
  const review = async () => {
    grid.current?.api.stopEditing();
    setBusy(true);
    setError("");
    try {
      await queue.current;
      const p = await invoke<Preview>("preview_translation_submission");
      setPreview(p);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  const signIn = async () => {
    setError("");
    try {
      const l = await invoke<Login>("begin_contribution_login");
      setLogin(l);
      await openUrl(l.verification_uri);
    } catch (e) {
      setError(String(e));
    }
  };
  useEffect(() => {
    if (!login) return;
    let cancelled = false,
      timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try {
        const status = await invoke<{ connected: boolean; interval: number }>(
          "poll_contribution_login"
        );
        if (cancelled) return;
        if (status.connected) {
          setAuth({ connected: true, configured: true });
          setLogin(null);
        } else timer = setTimeout(poll, status.interval * 1000);
      } catch (e) {
        if (!cancelled) {
          setError(String(e));
          setLogin(null);
        }
      }
    };
    timer = setTimeout(poll, login.interval * 1000);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [login]);
  const submit = async () => {
    if (!preview) return;
    setBusy(true);
    setError("");
    try {
      const result = await invoke<{ url: string; existing: boolean }>("submit_translation_draft", {
        revision: preview.revision,
        digest: preview.digest,
      });
      setPr(result.url);
      setMessage(
        result.existing
          ? "This submission already has a pull request"
          : "Draft pull request created"
      );
    } catch (e) {
      setError(String(e));
      void authReady();
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
      setError(`Could not save the draft: ${String(e)}. Fix the correction before closing.`);
    }
  };
  const count = Object.keys(edits).length;
  return (
    <Dialog
      open
      onOpenChange={(value) => {
        if (!value) void close();
      }}
    >
      <DialogContent className="translation-editor" onInteractOutside={(e) => e.preventDefault()}>
        <div className="translation-heading">
          <div>
            <span className="translation-eyebrow">COMMUNITY TRANSLATIONS · EXPERIMENT</span>
            <DialogTitle>The Drifter — improve the Hebrew translation</DialogTitle>
            <DialogDescription>
              Find a line, edit your correction, then review and send a draft pull request.
            </DialogDescription>
          </div>
        </div>
        <p className="translation-notice">
          This first POC edits Drifter’s stored text format: Hebrew is in display order. It does not
          install a patch or publish a release.
        </p>
        {error && (
          <p role="alert" className="translation-error">
            {error}
          </p>
        )}
        {draft && !preview && (
          <>
            <div className="translation-toolbar">
              <label className="translation-search">
                <Search size={16} />
                <input
                  aria-label="Search subtitles"
                  placeholder="Search English, Hebrew, speaker or scene…"
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
                Only my changes ({count})
              </label>
              <Button
                variant="outline"
                size="sm"
                onClick={() => grid.current?.api.undoCellEditing()}
                aria-label="Undo edit"
              >
                <Undo2 size={16} />
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => grid.current?.api.redoCellEditing()}
                aria-label="Redo edit"
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
        {preview && (
          <div className="translation-review">
            <h3>Review {preview.changes.length} correction(s)</h3>
            <p>
              Your changes will be public on GitHub. This experiment opens a draft PR marked “do not
              merge”.
            </p>
            {preview.changes.map((change) => (
              <article key={change.key}>
                <p>{change.english}</p>
                <div className="translation-before" dir="auto">
                  − {change.before}
                </div>
                <div className="translation-after" dir="auto">
                  + {change.after}
                </div>
              </article>
            ))}
            {!auth.connected && (
              <div className="translation-auth">
                <h3>Sign in to GitHub to submit</h3>
                <p>No Git installation or local repository is needed.</p>
                {!auth.configured && (
                  <output>
                    GitHub sign-in is not configured in this build. The developer must supply the
                    Kitor OAuth app client ID.
                  </output>
                )}
                {login ? (
                  <p>
                    Enter <strong className="translation-code">{login.user_code}</strong> on{" "}
                    <button type="button" onClick={() => void openUrl(login.verification_uri)}>
                      GitHub’s authorization page ↗
                    </button>
                    . Waiting for approval…
                  </p>
                ) : (
                  <Button disabled={!auth.configured} onClick={() => void signIn()}>
                    <Github size={16} /> Sign in with GitHub
                  </Button>
                )}
              </div>
            )}
            {pr && (
              <p className="translation-success">
                Ready for review.{" "}
                <button type="button" onClick={() => void openUrl(pr)}>
                  Open draft pull request ↗
                </button>
              </p>
            )}
          </div>
        )}
        <div className="translation-footer">
          <div aria-live="polite">
            <span className={saved ? "saved-dot" : "saving-dot"} /> {message}
            {draft && (
              <small>
                {draft.rows.length.toLocaleString()} lines · based on {draft.base_branch} /{" "}
                {draft.base_sha.slice(0, 7)}
              </small>
            )}
          </div>
          <div className="translation-footer-actions">
            {auth.connected && (
              <Button
                variant="ghost"
                disabled={busy}
                onClick={() =>
                  void invoke("disconnect_contribution_github")
                    .then(() => authReady())
                    .catch((e) => setError(String(e)))
                }
              >
                Disconnect GitHub
              </Button>
            )}
            {preview ? (
              <>
                <Button variant="outline" disabled={busy} onClick={() => setPreview(null)}>
                  Back to editing
                </Button>
                <Button
                  disabled={busy || !auth.connected || !preview.changes.length || !!pr}
                  onClick={() => void submit()}
                >
                  {busy ? "Submitting…" : "Create draft PR"}
                </Button>
              </>
            ) : (
              <Button disabled={!draft || busy || count === 0} onClick={() => void review()}>
                {busy ? "Preparing…" : `Review ${count} correction${count === 1 ? "" : "s"}`}
              </Button>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
