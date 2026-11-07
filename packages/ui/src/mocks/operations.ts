import { delay, HttpResponse, http } from "msw";

export type ReviewScenario =
  | "normal"
  | "failure"
  | "no-match"
  | "scan-warnings"
  | "missing-runtime"
  | "private-library"
  | "unsupported"
  | "up-to-date";
export type OperationResult = { progress: number; step: string; message: string };
const stages: Record<string, string[]> = {
  install: [
    "Downloading translation · 12 MB / 40 MB",
    "Backing up original files",
    "Installing translation",
    "Translation installed",
  ],
  update: [
    "Downloading translation update",
    "Checking original backup",
    "Applying updated files",
    "Translation updated",
  ],
  restore: [
    "Checking backup",
    "Restoring original files",
    "Verifying game",
    "Original game restored",
  ],
  steam: [
    "Waiting for Steam download",
    "Downloading Windows game files",
    "Extracting and detecting games",
    "Game added to library",
  ],
  gog: [
    "Downloading installer · 240 MB / 800 MB",
    "Extracting game files",
    "Detecting supported game",
    "Game added to library",
  ],
  scan: [
    "Scanning Steam folders",
    "Scanning GOG folders",
    "Skipping games already in library",
    "Added 1 game · scanned 4 locations",
  ],
  folder: [
    "Reading selected folder",
    "Identifying game",
    "Matching translation registry",
    "Recognized game added to library",
  ],
  "install-runtime": [
    "Downloading ScummVM",
    "Extracting runtime",
    "Verifying installation",
    "ScummVM ready",
  ],
  "update-runtime": [
    "Removing old runtime",
    "Downloading ScummVM",
    "Verifying installation",
    "ScummVM updated",
  ],
  "install-app": ["Downloading Kitor", "Verifying update", "Installing update", "Ready to restart"],
  refresh: [
    "Finding game folder",
    "Checking game identity",
    "Verifying patch files",
    "Game files found · patch verified",
  ],
};
export const operationHandlers = [
  http.post("*/api/review/operation", async ({ request }) => {
    const { action, stage, scenario, attempt } = (await request.json()) as {
      action: string;
      stage: number;
      scenario: ReviewScenario;
      attempt: number;
    };
    await delay(450);
    if (scenario === "failure" && stage === 1 && attempt === 0)
      return HttpResponse.json(
        {
          message:
            "The operation could not finish. Check your connection and try again. Your current library state has been kept.",
        },
        { status: 503 }
      );
    if (scenario === "no-match" && action === "folder")
      return HttpResponse.json(
        {
          message:
            "No patch match for this folder. Choose the game installation folder and try again.",
        },
        { status: 422 }
      );
    if (scenario === "private-library" && action === "refresh-steam")
      return HttpResponse.json(
        {
          message:
            "Steam game details are private. Make them public in Steam privacy settings, then refresh your library.",
        },
        { status: 403 }
      );
    const step = stages[action]?.[stage] ?? (stage < 3 ? "Working…" : "Complete");
    const messages: Record<string, string> = {
      play: "Game launched",
      remove: "Game removed from library. Files remain on disk.",
      "remove-all": "Library cleared. Game files remain on disk.",
      "open-folder": "Game folder opened",
      "open-runtime": "ScummVM opened",
      "runtime-folder": "ScummVM folder opened",
      "remove-runtime": "ScummVM uninstalled",
      "connect-steam": "Connected to Steam",
      "connect-gog": "Connected to GOG",
      "refresh-steam": "Steam library refreshed · 12 games",
      "refresh-gog": "GOG library refreshed · 8 games",
      "disconnect-steam": "Steam disconnected and saved key removed",
      "disconnect-gog": "Logged out of GOG",
      "steam-key": "Steam key saved",
      "check-app": scenario === "up-to-date" ? "Kitor is up to date" : "Kitor 1.1 is available",
      "check-patches": "Translation update found for Jenny LeClue",
      restart: "Kitor restarted · your library and settings were kept",
      report: "Email draft prepared. You send it from your email app.",
      "copy-report": "Report copied",
      "steam-privacy": "Steam privacy settings opened",
      "steam-api": "Steam API key page opened",
      "store-steam": "Steam store page opened",
      "store-gog": "GOG store page opened",
    };
    return HttpResponse.json({
      progress: (stage + 1) * 25,
      step,
      message: messages[action] ?? step,
    } satisfies OperationResult);
  }),
];
