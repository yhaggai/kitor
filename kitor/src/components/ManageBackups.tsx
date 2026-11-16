import { useTranslation } from "@kitor/ui";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpen, Trash2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useDialog } from "@/hooks/useDialog";

type Backup = {
  id: string;
  game: string;
  title: string;
  title_he?: string | null;
  target?: string | null;
  version?: string | null;
  bytes: number;
  needs_confirmation: boolean;
  can_delete: boolean;
};

export default function ManageBackups() {
  const { t, language } = useTranslation();
  const { alert, confirm } = useDialog();
  const [backups, setBackups] = useState<Backup[] | null>(null);
  const [busy, setBusy] = useState(false);

  const displaySize = (bytes: number) => {
    const units = ["B", "KiB", "MiB", "GiB", "TiB"];
    const unit = bytes ? Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), 4) : 0;
    return `${new Intl.NumberFormat(language, { maximumFractionDigits: 1 }).format(bytes / 1024 ** unit)} ${units[unit]}`;
  };

  const manage = async () => {
    if (backups) {
      setBackups(null);
      return;
    }
    setBusy(true);
    try {
      setBackups(await invoke<Backup[]>("list_game_backups"));
    } catch (error) {
      await alert({
        failure: true,
        title: t("Could Not Load Backups"),
        description: String(error),
      });
    } finally {
      setBusy(false);
    }
  };

  const open = async (backup: Backup) => {
    try {
      await invoke("open_backup_folder", { backupId: backup.id });
    } catch (error) {
      await alert({
        failure: true,
        title: t("Could Not Open Backup Folder"),
        description: String(error),
      });
    }
  };

  const remove = async (backup: Backup) => {
    setBusy(true);
    try {
      const confirmed = await confirm({
        title: t("Delete Backup?"),
        description: [
          t("Deleting this backup frees up {0}.", [displaySize(backup.bytes)]),
          ...(backup.needs_confirmation
            ? [
                t(
                  "If the game is still patched, you may need to repair or reinstall it to restore the original files or update the translation."
                ),
              ]
            : []),
        ].join(" "),
        confirmText: t("Delete Backup"),
        cancelText: t("Cancel"),
      });
      if (!confirmed) return;
      await invoke("delete_game_backup", {
        backupId: backup.id,
        confirmed: backup.needs_confirmation,
      });
      setBackups((current) => current?.filter((entry) => entry.id !== backup.id) ?? null);
    } catch (error) {
      await alert({
        failure: true,
        title: t("Could Not Delete Backup"),
        description: String(error),
      });
      try {
        setBackups(await invoke<Backup[]>("list_game_backups"));
      } catch {
        setBackups(null);
      }
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("Game Backups")}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <Button variant="outline" onClick={manage} disabled={busy} aria-expanded={backups !== null}>
          {t(backups ? "Hide Backups" : "Manage Backups")}
        </Button>
        {backups?.length === 0 && (
          <p className="text-sm text-muted-foreground">{t("No game backups found.")}</p>
        )}
        {backups && backups.length > 0 && (
          <ul className="divide-y rounded-md border">
            {backups.map((backup) => (
              <li key={backup.id} className="flex flex-wrap items-center justify-between gap-3 p-3">
                <div className="min-w-0">
                  <p className="font-medium">
                    {(language === "he" && backup.title_he) || backup.title}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    <bdi>
                      {[backup.version, displaySize(backup.bytes)].filter(Boolean).join(" · ")}
                    </bdi>
                  </p>
                  <p dir="auto" className="break-all text-xs text-muted-foreground">
                    {backup.target ?? backup.id}
                  </p>
                  {!backup.can_delete && (
                    <p className="text-sm text-muted-foreground">
                      {t("Finish patch recovery before deleting this backup.")}
                    </p>
                  )}
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button size="sm" variant="outline" disabled={busy} onClick={() => open(backup)}>
                    <FolderOpen className="size-4" /> {t("Open Folder")}
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    className="text-destructive"
                    disabled={busy || !backup.can_delete}
                    onClick={() => remove(backup)}
                  >
                    <Trash2 className="size-4" /> {t("Delete Backup")}
                  </Button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
