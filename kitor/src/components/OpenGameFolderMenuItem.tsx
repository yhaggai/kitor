import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpen } from "lucide-react";
import { useDialog } from "@/hooks/useDialog";

export function OpenGameFolderMenuItem({ gameId }: { gameId: number }) {
  const { alert } = useDialog();

  const handleOpenFolder = async () => {
    try {
      await invoke("open_game_folder", { gameId });
    } catch (error) {
      await alert({ title: "Could Not Open Game Folder", description: String(error) });
    }
  };

  return (
    <DropdownMenu.Item
      className="flex cursor-pointer items-center rounded-sm px-2 py-2 text-sm text-foreground outline-none hover:bg-accent focus:bg-accent"
      onSelect={handleOpenFolder}
    >
      <FolderOpen className="mr-2 h-4 w-4" />
      Open Game Folder
    </DropdownMenu.Item>
  );
}
