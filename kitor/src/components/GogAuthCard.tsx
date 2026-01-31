import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { CheckCircle, LogIn, LogOut, XCircle } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useDialog } from "@/hooks/useDialog";
import type { GogGame } from "@/store/store";
import { useAppStore } from "@/store/store";

export default function GogAuthCard() {
  const { gogAuth, gogGames, setGogAuth, setGogGames } = useAppStore();
  const alert = useDialog();
  const [isLoggingIn, setIsLoggingIn] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);

  const checkAuthStatus = useCallback(async () => {
    try {
      const isAuthenticated = await invoke<boolean>("check_gog_auth_status");
      setGogAuth({ isAuthenticated });

      if (isAuthenticated) {
        try {
          const games = await invoke<GogGame[]>("get_gog_user_games");
          setGogGames(games);
        } catch (error) {
          console.error("Failed to fetch GOG games:", error);
        }
      }
    } catch (error) {
      console.error("Failed to check GOG auth status:", error);
      setGogAuth({ isAuthenticated: false });
    }
  }, [setGogAuth, setGogGames]);

  useEffect(() => {
    checkAuthStatus();
  }, [checkAuthStatus]);

  useEffect(() => {
    const unlistenSuccess = listen("gog-login-success", () => {
      setIsLoggingIn(false);
      checkAuthStatus();
      alert.alert({ title: "Success", description: "Successfully logged in to GOG" });
    });

    const unlistenError = listen<string>("gog-login-error", (event) => {
      setIsLoggingIn(false);
      alert.alert({ title: "Login Failed", description: event.payload });
    });

    return () => {
      unlistenSuccess.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  }, [checkAuthStatus, alert]);

  const handleLogin = async () => {
    try {
      setIsLoggingIn(true);
      await invoke("login_to_gog");
    } catch (error) {
      console.error("Failed to initiate GOG login:", error);
      await alert.alert({ title: "Login Failed", description: String(error) });
      setIsLoggingIn(false);
    }
  };

  const handleLogout = async () => {
    const confirmed = await alert.confirm({
      title: "Logout from GOG?",
      description: "Are you sure you want to logout from GOG?",
      confirmText: "Logout",
      cancelText: "Cancel",
    });

    if (!confirmed) {
      return;
    }

    try {
      setIsLoggingOut(true);
      await invoke("logout_from_gog");
      setGogAuth({ isAuthenticated: false });
      setGogGames([]);
    } catch (error) {
      console.error("Failed to logout from GOG:", error);
      await alert.alert({ title: "Logout Failed", description: String(error) });
    } finally {
      setIsLoggingOut(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>GOG Integration</CardTitle>
        <CardDescription>
          Connect your GOG account to download games from your library
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center gap-3">
          {gogAuth.isAuthenticated ? (
            <>
              <CheckCircle className="h-5 w-5 text-green-500" />
              <div className="flex-1">
                <p className="font-medium">Connected to GOG</p>
                <p className="text-sm text-muted-foreground">
                  {gogGames.length} game{gogGames.length !== 1 ? "s" : ""} in library
                </p>
              </div>
              <Badge variant="secondary">Connected</Badge>
            </>
          ) : (
            <>
              <XCircle className="h-5 w-5 text-muted-foreground" />
              <div className="flex-1">
                <p className="font-medium">Not Connected</p>
                <p className="text-sm text-muted-foreground">Login to access your GOG library</p>
              </div>
            </>
          )}
        </div>

        {gogAuth.isAuthenticated ? (
          <Button
            onClick={handleLogout}
            variant="outline"
            disabled={isLoggingOut}
            className="w-full"
          >
            <LogOut className="h-4 w-4 mr-2" />
            {isLoggingOut ? "Logging out..." : "Logout from GOG"}
          </Button>
        ) : (
          <Button onClick={handleLogin} disabled={isLoggingIn} className="w-full">
            <LogIn className="h-4 w-4 mr-2" />
            {isLoggingIn ? "Waiting for login..." : "Login to GOG"}
          </Button>
        )}

        {gogAuth.isAuthenticated && gogGames.length > 0 && (
          <div className="border-t pt-4">
            <p className="text-sm font-medium mb-2">Your GOG Games:</p>
            <div className="space-y-1 max-h-40 overflow-y-auto">
              {gogGames.map((game) => (
                <div
                  key={game.id}
                  className="text-sm text-muted-foreground flex items-center gap-2"
                >
                  <span className="truncate">{game.title}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
