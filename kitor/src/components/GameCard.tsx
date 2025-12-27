import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Play, Package } from "lucide-react";
import type { Game } from "@/store/store";

interface GameCardProps {
  game: Game;
}

export default function GameCard({ game }: GameCardProps) {
  const handleLaunch = async () => {
    try {
      await invoke("launch_game", { gameId: game.id });
    } catch (error) {
      console.error("Failed to launch game:", error);
      alert(`Failed to launch game: ${error}`);
    }
  };

  return (
    <Card className="hover:shadow-lg transition-shadow">
      <CardContent className="p-0">
        <div className="aspect-[3/4] bg-gradient-to-br from-primary/20 to-secondary/20 rounded-t-lg flex items-center justify-center">
          {game.cover_url ? (
            <img
              src={game.cover_url}
              alt={game.title}
              className="w-full h-full object-cover rounded-t-lg"
            />
          ) : (
            <Package className="h-16 w-16 text-muted-foreground" />
          )}
        </div>
      </CardContent>
      <CardFooter className="flex flex-col gap-2 p-4">
        <div className="w-full">
          <h3 className="font-semibold text-sm truncate mb-1" title={game.title}>
            {game.title}
          </h3>
          <Badge variant="secondary" className="text-xs">
            {game.scummvm_id}
          </Badge>
        </div>
        <Button className="w-full" size="sm" onClick={handleLaunch}>
          <Play className="mr-2 h-3 w-3" />
          Play
        </Button>
      </CardFooter>
    </Card>
  );
}



