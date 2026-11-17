type Baseline = {
  platform: string;
  evidence: string;
  language: string;
  patchVersion: string;
  build_id?: string;
  manifest_id?: string;
  depot_id?: string;
  released_on?: string;
  first_seen_on?: string;
  release_date_source?: string;
  status: "match" | "different" | "unavailable";
  fileCheck?:
    | { status: "unchanged"; filesChecked: number }
    | { status: "changed"; paths: string[] }
    | { status: "unavailable"; reason: string };
};

type BuildRow = {
  gameId: string;
  store: "steam" | "gog";
  os?: "windows" | "osx" | "linux";
  latest: { buildId: string; version?: string; releasedOn?: string } | null;
  availability: "ok" | "error" | "unavailable";
  lastSuccessAt: string | null;
  change: { previousBuildId: string; buildId: string; detectedAt: string } | null;
  supported: Baseline[];
};

const platform = { windows: "Windows", osx: "macOS", linux: "Linux" };
const date = (value: string) => new Date(value).toLocaleString();
const needsReview = (row: BuildRow) =>
  row.change !== null || row.supported.some((baseline) => baseline.status === "different");

function line(parent: HTMLElement, value: string, className = "") {
  const element = document.createElement("div");
  element.textContent = value;
  element.className = className;
  parent.append(element);
  return element;
}

function link(parent: HTMLElement, label: string, url: string) {
  const anchor = document.createElement("a");
  anchor.textContent = label;
  anchor.href = url;
  parent.append(anchor);
}

function renderStore(row: BuildRow) {
  const element = document.createElement("div");
  element.dataset.store = row.store;
  const name = row.store === "steam" ? "Steam" : `GOG · ${platform[row.os!]}`;
  const label = document.createElement("strong");
  label.textContent = name;
  element.append(label);

  const current = row.latest;
  if (current) {
    line(element, `Build ${current.buildId}`);
    if (current.version) line(element, current.version, "build-meta");
    if (current.releasedOn) line(element, `Released ${current.releasedOn}`, "build-meta");
  }

  if (row.availability !== "ok") {
    line(element, row.availability === "error" ? "Store lookup failed" : "No public build", "attention");
    if (row.lastSuccessAt) line(element, `Last seen ${date(row.lastSuccessAt)}`, "build-meta");
  } else if (row.supported.some(({ fileCheck }) => fileCheck?.status === "changed")) {
    line(element, "↑ Patch files changed · verification needed", "attention");
  } else if (row.supported.some(({ fileCheck }) => fileCheck?.status === "unchanged")
    && row.supported.every(({ status, fileCheck }) => status === "match" || fileCheck?.status === "unchanged")) {
    line(element, "✓ Compared patch files unchanged", "build-match");
  } else if (row.supported.some((baseline) => baseline.status === "different")) {
    line(element, "↑ Build differs from patch baseline", "attention");
  } else if (row.supported.length && row.supported.every((baseline) => baseline.status === "match")) {
    line(element, "✓ Matches patch baseline", "build-match");
  } else {
    line(element, row.supported.length ? "Comparison unavailable" : "Supported build unrecorded", "build-meta");
  }

  if (row.change) {
    const change = row.change.previousBuildId === row.change.buildId
      ? "Depot manifest changed"
      : `Changed from build ${row.change.previousBuildId}`;
    line(element, change, "attention").title = date(row.change.detectedAt);
  }

  if (row.supported.length) {
    const details = document.createElement("details");
    const summary = document.createElement("summary");
    summary.textContent = "Patch build details";
    details.append(summary);
    const status = { match: "Matches", different: "Differs", unavailable: "Comparison unavailable" };
    for (const baseline of row.supported) {
      const os = platform[baseline.platform as keyof typeof platform] ?? baseline.platform;
      const id = baseline.manifest_id
        ? `Depot ${baseline.depot_id} · manifest ${baseline.manifest_id}`
        : `Build ${baseline.build_id}`;
      line(details, `${os} · ${status[baseline.status]}`);
      line(details, id, "build-meta");
      const check = baseline.fileCheck;
      if (check?.status === "unchanged") line(details, `${check.filesChecked} patch inputs unchanged (${row.store === "steam" ? "Steam SHA-1" : "GOG manifest hashes"})`, "build-match");
      if (check?.status === "changed") {
        line(details, "Patch files changed or missing", "attention");
        for (const path of check.paths) line(details, path);
      }
      if (check?.status === "unavailable") line(details, check.reason, "build-meta");
      if (baseline.patchVersion) line(details, `Patch ${baseline.patchVersion}`, "build-meta");
      const datedEntry = line(details, baseline.released_on ? `Released ${baseline.released_on} `
        : baseline.first_seen_on ? `First seen ${baseline.first_seen_on} ` : "Release date unknown", "build-meta");
      if (baseline.release_date_source) link(datedEntry, "Date source", baseline.release_date_source);
      link(details, "Evidence", `https://github.com/yhaggai/kitor/blob/HEAD/${baseline.evidence}`);
    }
    element.append(details);
  }
  return element;
}

export async function showBuildMonitor(root: HTMLElement) {
  const status = root.querySelector<HTMLElement>("#scan-status")!;
  const search = root.querySelector<HTMLInputElement>("#search")!;
  const store = root.querySelector<HTMLSelectElement>("#store")!;
  const review = root.querySelector<HTMLInputElement>("#review")!;
  const empty = root.querySelector<HTMLElement>("#empty")!;
  const cards = [...root.querySelectorAll<HTMLElement>("[data-game]")];
  const rowsByGame = new Map<string, BuildRow[]>();

  const filter = () => {
    const query = search.value.trim().toLowerCase();
    let count = 0;
    for (const card of cards) {
      const rows = rowsByGame.get(card.dataset.game!) ?? [];
      const summaries = card.querySelectorAll<HTMLElement>("[data-store]");
      for (const summary of summaries) {
        summary.hidden = Boolean(store.value && store.value !== summary.dataset.store);
      }
      card.hidden = !card.dataset.title!.toLowerCase().includes(query)
        || ![...summaries].some((summary) => !summary.hidden)
        || (review.checked && !rows.some((row) => (!store.value || store.value === row.store) && needsReview(row)));
      if (!card.hidden) count++;
    }
    empty.hidden = count > 0;
  };
  root.querySelector(".filters")!.addEventListener("input", filter);
  filter();

  try {
    const response = await fetch(root.dataset.snapshot!, { cache: "no-cache" });
    if (response.status === 404) {
      status.textContent = "Awaiting the first Steam and GOG store scan.";
      return;
    }
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const { checkedAt, rows }: { checkedAt: string; rows: BuildRow[] } = await response.json();
    for (const row of rows) {
      const current = rowsByGame.get(row.gameId) ?? [];
      current.push(row);
      rowsByGame.set(row.gameId, current);
    }
    for (const card of cards) {
      const rows = rowsByGame.get(card.dataset.game!);
      if (rows) card.querySelector(".store-builds")!.replaceChildren(...rows.map(renderStore));
    }
    status.textContent = `Last scan ${date(checkedAt)} · ${rows.filter(needsReview).length} store checks with changes or differences`;
    review.disabled = false;
    filter();
  } catch {
    status.textContent = "The store scan could not be loaded. Refresh to try again.";
  }
}
