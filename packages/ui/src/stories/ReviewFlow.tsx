import { useEffect, useRef, useState } from "react";
import { Button, FlowDialog, OperationProgress } from "../index";
import type { OperationResult, ReviewScenario } from "../mocks/operations";

export type FlowRequest = {
  action: string;
  title: string;
  description?: string;
  gameId?: string;
  choice?: string;
};
export function ReviewFlow({
  request,
  scenario,
  connectedGog,
  onDone,
  onClose,
  onBusy,
}: {
  request: FlowRequest;
  scenario: ReviewScenario;
  connectedGog: boolean;
  onDone: (request: FlowRequest, message: string) => void;
  onClose: () => void;
  onBusy: (busy: boolean) => void;
}) {
  const [phase, setPhase] = useState<"confirm" | "busy" | "done" | "error">("confirm");
  const [progress, setProgress] = useState<OperationResult>();
  const [error, setError] = useState("");
  const [attempt, setAttempt] = useState(0);
  const [folder, setFolder] = useState("");
  const [credential, setCredential] = useState("");
  const [details, setDetails] = useState("");
  const [reportGame, setReportGame] = useState("");
  const [diagnostics, setDiagnostics] = useState(false);
  const [files, setFiles] = useState<File[]>([]);
  const [hidden, setHidden] = useState(false);
  const controller = useRef<AbortController | null>(null);
  useEffect(() => () => controller.current?.abort(), []);
  const action = request.action;
  const download = action === "steam" || action === "gog";
  const needsFolder = download || action === "folder";
  const secret = action === "steam-key";
  const auth = action === "connect-steam" || action === "connect-gog";
  const isReport = action === "report";
  const destructive = [
    "restore",
    "remove",
    "remove-all",
    "remove-runtime",
    "disconnect-gog",
    "disconnect-steam",
  ].includes(action);
  const blocked =
    (needsFolder && !folder) ||
    (secret && !credential.trim()) ||
    (isReport && !details.trim()) ||
    (action === "gog" && !connectedGog) ||
    (download && scenario === "unsupported");
  async function run(override?: string) {
    controller.current?.abort();
    const abort = new AbortController();
    controller.current = abort;
    setPhase("busy");
    onBusy(true);
    setError("");
    setProgress({
      progress: 0,
      step: auth ? "Waiting for browser sign-in…" : "Starting…",
      message: "",
    });
    try {
      let result: OperationResult | undefined;
      for (let stage = 0; stage < 4; stage++) {
        const response = await fetch("/api/review/operation", {
          method: "POST",
          signal: abort.signal,
          headers: { "Content-Type": "application/json" },
          // Never send entered credentials, report text or file contents from the review.
          body: JSON.stringify({ action: override ?? action, stage, scenario, attempt }),
        });
        result = (await response.json()) as OperationResult;
        if (!response.ok) throw new Error(result.message);
        setProgress(result);
      }
      setPhase("done");
      onBusy(false);
      onDone({ ...request, action: override ?? action }, result?.message ?? "Complete");
    } catch (reason) {
      if (!abort.signal.aborted) {
        setError(String((reason as Error).message));
        setPhase("error");
        setAttempt((value) => value + 1);
        onBusy(false);
      }
    }
  }
  function addFiles(incoming: File[]) {
    const next = [...files, ...incoming];
    if (
      next.length > 5 ||
      next.reduce((sum, file) => sum + file.size, 0) > 10 * 1024 * 1024 ||
      next.some((file) => !["image/png", "image/jpeg", "image/webp"].includes(file.type))
    ) {
      setError("Choose up to 5 PNG, JPEG or WebP images, 10 MB total.");
      return;
    }
    setError("");
    setFiles(next);
  }
  if (hidden)
    return (
      <div className="kt-notice">
        <output>
          {phase === "busy"
            ? `${request.title} · ${progress?.step}`
            : phase === "error"
              ? error
              : progress?.message}
        </output>{" "}
        <Button variant="secondary" onClick={() => setHidden(false)}>
          Show progress
        </Button>
      </div>
    );
  return (
    <FlowDialog
      title={phase === "done" ? "Complete" : request.title}
      description={request.description}
      dismissible={phase !== "busy" || download || auth}
      onClose={() => {
        if (phase === "busy" && download) setHidden(true);
        else {
          controller.current?.abort();
          onBusy(false);
          onClose();
        }
      }}
      footer={
        phase === "busy" ? (
          download ? (
            <Button variant="secondary" onClick={() => setHidden(true)}>
              Hide download
            </Button>
          ) : auth ? (
            <Button
              variant="secondary"
              onClick={() => {
                controller.current?.abort();
                onBusy(false);
                onClose();
              }}
            >
              Cancel sign-in
            </Button>
          ) : undefined
        ) : phase === "done" ? undefined : (
          <>
            {isReport && (
              <Button
                variant="secondary"
                disabled={!!blocked}
                onClick={() => void run("copy-report")}
              >
                Copy report
              </Button>
            )}
            <Button
              className={destructive ? "kt-danger" : ""}
              disabled={!!blocked}
              onClick={() => void run()}
            >
              {phase === "error"
                ? "Try again"
                : isReport
                  ? files.length || diagnostics
                    ? "Save email with attachments"
                    : "Prepare email"
                  : auth
                    ? "Continue to sign in"
                    : destructive
                      ? "Confirm"
                      : "Continue"}
            </Button>
          </>
        )
      }
    >
      {phase === "busy" ? (
        <>
          <OperationProgress value={progress?.progress ?? 0} step={progress?.step ?? "Starting…"} />
          {download && <p>Keep Kitor open. Hiding this window does not cancel the download.</p>}
        </>
      ) : phase === "done" ? (
        <output>{progress?.message}</output>
      ) : (
        <>
          {needsFolder && (
            <>
              <label className="kt-field">
                {action === "folder" ? "Game installation folder" : "Install location"}
                <input
                  value={folder}
                  onChange={(event) => setFolder(event.target.value)}
                  placeholder="Choose a folder"
                />
              </label>
              <Button
                variant="secondary"
                onClick={() =>
                  setFolder(action === "folder" ? "/Games/Day of the Tentacle" : "/Games/Kitor")
                }
              >
                Choose folder
              </Button>
              {folder && (
                <p>
                  {folder}
                  {download ? `/${request.gameId}-${action}` : ""}
                </p>
              )}
            </>
          )}
          {action === "steam" && (
            <p>
              Sign in to the Steam client with an account that owns the game. Kitor downloads the
              supported Windows files, verifies them and detects games in the collection. Install
              translations separately.
            </p>
          )}
          {action === "gog" && (
            <p>
              {connectedGog
                ? "Your GOG account owns this game. Download and extract its Windows installer; the game will appear in your library. Install translations separately."
                : "Connect the GOG account that owns this game in Settings before continuing."}
            </p>
          )}
          {download && scenario === "unsupported" && (
            <p role="alert">
              This edition cannot be downloaded through Kitor. Open the store, install a supported
              edition, then scan its folder.
            </p>
          )}
          {secret && (
            <>
              <label className="kt-field">
                Steam Web API key
                <input
                  type="password"
                  autoComplete="off"
                  value={credential}
                  onChange={(event) => setCredential(event.target.value)}
                  placeholder="Use any sample value for review"
                />
              </label>
              <p>
                In the desktop app, credentials are saved in your system credential store. Use
                sample text in this preview.
              </p>
            </>
          )}
          {auth && (
            <p>
              Sign-in opens in the service’s browser window. This preview simulates returning to
              Kitor after authorization.
            </p>
          )}
          {isReport && (
            <div
              className="kt-flow-body"
              onPaste={(event) => {
                if (event.clipboardData.files.length) {
                  event.preventDefault();
                  addFiles(Array.from(event.clipboardData.files));
                }
              }}
            >
              <label className="kt-field">
                Game / patch (optional)
                <input
                  value={reportGame}
                  onChange={(event) => setReportGame(event.target.value)}
                  maxLength={200}
                />
              </label>
              <label className="kt-field">
                What went wrong?
                <textarea
                  value={details}
                  onChange={(event) => setDetails(event.target.value)}
                  maxLength={12000}
                  placeholder="Steps to reproduce, expected behavior, actual behavior and exact error."
                />
              </label>
              <label className="kt-field">
                Add screenshots
                <input
                  type="file"
                  accept="image/png,image/jpeg,image/webp"
                  multiple
                  onChange={(event) => addFiles(Array.from(event.target.files ?? []))}
                />
              </label>
              <p>Paste or choose up to 5 images, 10 MB total.</p>
              {files.map((file, index) => (
                <div key={`${file.name}-${index}`}>
                  {file.name}{" "}
                  <Button
                    variant="ghost"
                    onClick={() => setFiles(files.filter((_, i) => i !== index))}
                  >
                    Remove {file.name}
                  </Button>
                </div>
              ))}
              <label className="kt-check">
                <input
                  type="checkbox"
                  checked={diagnostics}
                  onChange={(event) => setDiagnostics(event.target.checked)}
                />
                Include diagnostics
              </label>
              {diagnostics && (
                <details>
                  <summary>Preview diagnostics</summary>
                  <pre>
                    Demo diagnostics: game IDs, patch status, runtime version. No personal paths or
                    credentials.
                  </pre>
                </details>
              )}
              <p>Kitor version and operating system are included. You send the email yourself.</p>
            </div>
          )}
        </>
      )}
      {error && (
        <p role="alert" className="kt-error">
          {error}
        </p>
      )}
    </FlowDialog>
  );
}
