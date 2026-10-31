import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";

export function GameInstallFolder({
  value,
  onChange,
  folders,
}: {
  value: string;
  onChange: (path: string) => void;
  folders: string[];
}) {
  const [error, setError] = useState<string | null>(null);
  const [browsing, setBrowsing] = useState(false);
  useEffect(() => {
    if (value) return;
    let active = true;
    void invoke<string>("get_game_install_root")
      .then((path) => {
        if (active) onChange(path);
      })
      .catch((error) => {
        if (active) setError(String(error));
      });
    return () => {
      active = false;
    };
  }, [value, onChange]);
  async function browse() {
    setBrowsing(true);
    setError(null);
    try {
      const path = await open({
        directory: true,
        multiple: false,
        title: "Choose game installation folder",
        ...(value ? { defaultPath: value } : {}),
      });
      if (path) onChange(path);
    } catch (error) {
      setError(String(error));
    } finally {
      setBrowsing(false);
    }
  }
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between gap-3">
        <span className="text-sm font-medium">Installation folder</span>
        <Button variant="outline" size="sm" disabled={browsing} onClick={() => void browse()}>
          {browsing ? "Opening…" : "Browse…"}
        </Button>
      </div>
      <div className="rounded-md border border-border bg-muted/40 text-foreground p-3 text-xs break-all font-mono">
        {value
          ? folders.map((folder) => (
              <p key={folder}>
                {value.replace(/\/$/, "")}/{folder}
              </p>
            ))
          : "Loading installation folder…"}
      </div>
      <p className="text-xs text-muted-foreground">
        {folders.length === 1
          ? "Kitor will create this folder for the download."
          : "Kitor will create a separate folder for each game here."}
      </p>
      {error && (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      )}
    </div>
  );
}
