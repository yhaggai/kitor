import { useTranslation } from "@kitor/ui";
import { useState } from "react";
import { refreshLibraryOnStartup, setRefreshLibraryOnStartup } from "@/store/libraryRefresh";

export function LibraryRefreshPreference({ provider }: { provider: "steam" | "gog" }) {
  const { t } = useTranslation();
  const [enabled, setEnabled] = useState(() => refreshLibraryOnStartup(provider));
  return (
    <label className="flex items-center gap-2 text-sm">
      <input
        type="checkbox"
        checked={enabled}
        onChange={(event) => {
          const checked = event.target.checked;
          setRefreshLibraryOnStartup(provider, checked);
          setEnabled(checked);
        }}
      />
      {t("Refresh the {0} library automatically when Kitor opens", [
        provider === "steam" ? "Steam" : "GOG",
      ])}
    </label>
  );
}
