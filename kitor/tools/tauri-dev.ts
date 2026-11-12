import { identifier } from "../src-tauri/tauri.conf.json";

const env = { ...process.env };

if (process.platform === "darwin") {
  const identity = "Kitor Local Development";
  const identities = Bun.spawnSync(["/usr/bin/security", "find-identity", "-p", "codesigning"]);

  if (identities.stdout.toString().includes(`"${identity}"`)) {
    env.KITOR_DEV_SIGNING_IDENTITY = identity;
    env.KITOR_DEV_SIGNING_IDENTIFIER = identifier;
    // Tauri runs Cargo from src-tauri, including on each Rust reload.
    const runner = "/bin/sh ../tools/run-signed-dev.sh";
    env.CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER ??= runner;
    env.CARGO_TARGET_X86_64_APPLE_DARWIN_RUNNER ??= runner;
  }
}

const child = Bun.spawn(["tauri", "dev", ...process.argv.slice(2)], {
  env,
  stdin: "inherit",
  stdout: "inherit",
  stderr: "inherit",
});
process.exitCode = await child.exited;
