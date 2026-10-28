#!/usr/bin/env python3
"""Build a complete signed app release and promote its manifest to the stable feed.

Requires the GitHub CLI for release operations. No credentials enter manifests.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[2]
PLATFORMS = {
    "darwin-aarch64": ("aarch64-apple-darwin", "macos", ".app.tar.gz"),
    "darwin-x86_64": ("x86_64-apple-darwin", "macos", ".app.tar.gz"),
    "windows-x86_64": ("x86_64-pc-windows-msvc", "nsis", "-setup.exe"),
}
REPO = "yhaggai/kitor"
FEED_TAG = "app-updates"


def version_tuple(value):
    if not re.fullmatch(r"(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)", value):
        raise ValueError("App releases require a stable X.Y.Z version")
    return tuple(map(int, value.split(".")))


def configured_version():
    config = json.loads((ROOT / "kitor/src-tauri/tauri.conf.json").read_text())
    package = json.loads((ROOT / "kitor/package.json").read_text())
    cargo = tomllib.loads((ROOT / "kitor/src-tauri/Cargo.toml").read_text())
    version = config["version"]
    version_tuple(version)
    if package["version"] != version or cargo["package"]["version"] != version:
        raise ValueError("Set the same version in tauri.conf.json, Cargo.toml, and package.json")
    return version


def gh(*args):
    return subprocess.check_output(["gh", *args], text=True).strip()


def api(path):
    return json.loads(gh("api", f"repos/{REPO}/{path}"))


def release(tag):
    return api(f"releases/tags/{tag}")


def filename(version, platform):
    return f"kitor-{version}-{platform}{PLATFORMS[platform][2]}"


def asset_map(data):
    assets = {asset["name"]: asset for asset in data["assets"]}
    if len(assets) != len(data["assets"]):
        raise ValueError("Duplicate release asset names")
    return assets


def manifest_for(data, signatures):
    version = data["tag_name"].removeprefix("app-v")
    version_tuple(version)
    if data["tag_name"] != f"app-v{version}" or data["prerelease"]:
        raise ValueError("Expected a stable app release")
    assets = asset_map(data)
    platforms = {}
    for platform in PLATFORMS:
        name = filename(version, platform)
        asset = assets[name]
        signature = signatures[name + ".sig"].strip()
        if not signature or asset["size"] <= 0:
            raise ValueError("Missing update content or signature")
        # The bundled updater verifies the signature before installing any bytes.
        expected = f"https://github.com/{REPO}/releases/download/app-v{version}/{name}"
        if asset["browser_download_url"] != expected:
            raise ValueError("Unexpected app update download URL")
        platforms[platform] = {"url": expected, "signature": signature}
    result = {"version": version, "notes": data.get("body") or "", "platforms": platforms}
    if data.get("published_at"):
        result["pub_date"] = data["published_at"]
    return result


def fetch_signatures(tag, directory):
    gh("release", "download", tag, "--repo", REPO, "--pattern", "*.sig", "--dir", str(directory))
    return {p.name: p.read_text() for p in directory.glob("*.sig")}


def upload_manifest(tag, manifest, directory):
    path = directory / "latest.json"
    path.write_text(json.dumps(manifest, indent=2) + "\n")
    gh("release", "upload", tag, str(path), "--repo", REPO, "--clobber")


def prepare():
    version = configured_version()
    tag = f"app-v{version}"
    # Refuse overwrites, including partially built drafts; retry failed jobs in
    # the original workflow or delete the draft explicitly before rebuilding.
    existing = json.loads(gh("api", f"repos/{REPO}/releases", "--paginate", "--slurp"))
    if any(r["tag_name"] == tag for page in existing for r in page):
        raise ValueError(f"Release {tag} already exists")
    sha = os.environ["GITHUB_SHA"]
    gh("release", "create", tag, "--repo", REPO, "--target", sha, "--draft", "--title", f"Kitor {version}",
       "--notes", "Release build in progress. Do not publish until the release workflow succeeds.")
    with open(os.environ["GITHUB_OUTPUT"], "a") as output:
        output.write(f"tag={tag}\n")


def upload(platform):
    version = configured_version()
    tag = f"app-v{version}"
    if not release(tag)["draft"]:
        raise ValueError("Only draft releases can receive build artifacts")
    target, bundle, extension = PLATFORMS[platform]
    folder = ROOT / "kitor/src-tauri/target" / target / "release/bundle"
    candidates = list((folder / bundle).glob("*" + extension))
    if len(candidates) != 1:
        raise ValueError(f"Expected one updater artifact for {platform}")
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        artifact = tmp / filename(version, platform)
        shutil.copyfile(candidates[0], artifact)
        signature = Path(str(candidates[0]) + ".sig")
        shutil.copyfile(signature, Path(str(artifact) + ".sig"))
        if platform.startswith("darwin-"):
            images = list((folder / "dmg").glob("*.dmg"))
            if len(images) != 1:
                raise ValueError("Expected one Mac installer")
            shutil.copyfile(images[0], tmp / f"kitor-{version}-{platform}.dmg")
        gh("release", "upload", tag, *(str(p) for p in tmp.iterdir()), "--repo", REPO, "--clobber")


def finalize():
    tag = f"app-v{configured_version()}"
    data = release(tag)
    if not data["draft"]:
        raise ValueError("Only finalize a draft release")
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        manifest = manifest_for(data, fetch_signatures(tag, tmp))
        upload_manifest(tag, manifest, tmp)
    gh("release", "edit", tag, "--repo", REPO, "--notes",
       "Installers and signed updates are ready for macOS (Apple Silicon and Intel) and Windows (x64).\n\nReplace these notes with the changes in this version before publishing. Publishing this release updates Kitor's stable app-update feed.")


def publish_feed(tag):
    if not re.fullmatch(r"app-v\d+\.\d+\.\d+", tag):
        raise ValueError("Not an app release tag")
    data = release(tag)
    if data["draft"] or data["prerelease"]:
        raise ValueError("Only published stable app releases can update the feed")
    # The feed is a dedicated prerelease so game-patch releases never replace it.
    releases = json.loads(gh("api", f"repos/{REPO}/releases", "--paginate", "--slurp"))
    feed = next((r for page in releases for r in page if r["tag_name"] == FEED_TAG), None)
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        manifest = manifest_for(data, fetch_signatures(tag, tmp))
        if feed:
            old_assets = asset_map(feed)
            if "latest.json" in old_assets:
                previous = tmp / "previous"
                previous.mkdir()
                gh("release", "download", FEED_TAG, "--repo", REPO, "--pattern", "latest.json", "--dir", str(previous))
                old = json.loads((previous / "latest.json").read_text())
                if version_tuple(old["version"]) > version_tuple(manifest["version"]):
                    raise ValueError("Refusing to move the stable feed to an older version")
        else:
            gh("release", "create", FEED_TAG, "--repo", REPO, "--target", os.environ["GITHUB_SHA"], "--prerelease",
               "--title", "Kitor app update feed", "--notes", "Stable app update metadata. Installers are attached to versioned app-v releases.")
        upload_manifest(FEED_TAG, manifest, tmp)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["validate", "prepare", "upload", "finalize", "publish-feed"])
    parser.add_argument("--platform", choices=PLATFORMS)
    parser.add_argument("--tag")
    args = parser.parse_args()
    if args.command == "validate":
        print(configured_version())
    elif args.command == "prepare":
        prepare()
    elif args.command == "upload":
        if not args.platform: parser.error("--platform is required")
        upload(args.platform)
    elif args.command == "finalize":
        finalize()
    else:
        if not args.tag: parser.error("--tag is required")
        publish_feed(args.tag)
