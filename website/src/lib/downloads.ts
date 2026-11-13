export const releasesUrl = "https://github.com/yhaggai/kitor/releases";

export const installers = [
  { id: "windows-x86_64", label: "Windows (x64)", suffix: "-setup.exe" },
  { id: "darwin-aarch64", label: "macOS (Apple Silicon / ARM64)", suffix: ".dmg" },
  { id: "darwin-x86_64", label: "macOS (Intel)", suffix: ".dmg" },
] as const;

type InstallerId = (typeof installers)[number]["id"];
// Keep this fallback on a verified stable desktop release for offline or rate-limited visitors.
const fallbackVersion = "0.1.3";
export const fallbackReleaseUrl = `${releasesUrl}/tag/app-v${fallbackVersion}`;
export function fallbackInstallerUrl(id: InstallerId) {
  const { suffix } = installers.find((installer) => installer.id === id)!;
  return `${releasesUrl}/download/app-v${fallbackVersion}/kitor-${fallbackVersion}-${id}${suffix}`;
}

type Browser = Pick<Navigator, "userAgent" | "maxTouchPoints"> & {
  userAgentData?: {
    platform?: string;
    mobile?: boolean;
    getHighEntropyValues?: (
      hints: string[]
    ) => Promise<{ architecture?: string; bitness?: string }>;
  };
};

export async function detectDownload(
  browser: Browser
): Promise<InstallerId | "mac" | "windows" | undefined> {
  const { userAgent, userAgentData, maxTouchPoints } = browser;
  if (
    userAgentData?.mobile ||
    /Android|iPhone|iPad|iPod|Windows Phone/i.test(userAgent) ||
    (/Macintosh/i.test(userAgent) && maxTouchPoints > 1)
  )
    return;

  const platform = userAgentData?.platform?.toLowerCase();
  const mac = platform ? platform === "macos" : /Macintosh/i.test(userAgent);
  const windows = platform ? platform === "windows" : /Windows NT/i.test(userAgent);
  if (!mac && !windows) return;

  const hints = await userAgentData
    ?.getHighEntropyValues?.(["architecture", "bitness"])
    .catch(() => undefined);
  if (mac) {
    // Apple Silicon browsers can report "Intel Mac" in the legacy user agent.
    if (hints?.architecture === "arm") return "darwin-aarch64";
    if (hints?.architecture === "x86") return "darwin-x86_64";
    return "mac";
  }
  if (hints?.architecture === "arm" || hints?.bitness === "32") return "windows";
  return "windows-x86_64";
}

type Release = {
  tag_name: string;
  draft: boolean;
  prerelease: boolean;
  assets: { name: string; browser_download_url: string }[];
};

function isNewer(version: number[], previous: number[]) {
  const index = version.findIndex((part, i) => part !== previous[i]);
  return index !== -1 && version[index] > previous[index];
}

export async function latestInstallers() {
  // Patch releases share the repository, so GitHub's generic /latest is unsuitable.
  let url = "https://api.github.com/repos/yhaggai/kitor/releases?per_page=100";
  let latest: { version: number[]; urls: Map<InstallerId, string> } | undefined;
  const signal = AbortSignal.timeout(6000);
  while (url) {
    const response = await fetch(url, { signal });
    if (!response.ok) return;
    for (const release of (await response.json()) as Release[]) {
      const match = /^app-v(\d+)\.(\d+)\.(\d+)$/.exec(release.tag_name);
      if (!match || release.draft || release.prerelease) continue;
      const version = match.slice(1).map(Number);
      if (latest && !isNewer(version, latest.version)) continue;

      const urls = new Map<InstallerId, string>();
      for (const { id, suffix } of installers) {
        const name = `kitor-${version.join(".")}-${id}${suffix}`;
        const expected = `${releasesUrl}/download/${release.tag_name}/${name}`;
        if (
          release.assets.some(
            (asset) => asset.name === name && asset.browser_download_url === expected
          )
        ) {
          urls.set(id, expected);
        }
      }
      if (urls.size === installers.length) latest = { version, urls };
    }
    url =
      response.headers
        .get("Link")
        ?.match(/<(https:\/\/api\.github\.com\/[^>]+)>; rel="next"/)?.[1] ?? "";
  }
  return latest;
}
