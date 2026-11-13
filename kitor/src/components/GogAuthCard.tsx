import { ManageMenu, useTranslation } from "@kitor/ui";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { CheckCircle, Gamepad2, LogIn, LogOut, XCircle } from "lucide-react";
import { useEffect, useState } from "react";
import BugReportDialog from "@/components/BugReportDialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useDialog } from "@/hooks/useDialog";
import { useGogLibrary } from "@/store/gog";
import { useAppStore } from "@/store/store";

export default function GogAuthCard() {
  const { t } = useTranslation();
  const { gogAuth, gogGames } = useAppStore();
  const alert = useDialog();
  const [isLoggingIn, setIsLoggingIn] = useState(false);
  const [isLoggingOut, setIsLoggingOut] = useState(false);

  const {
    loading: isLoadingGames,
    error: libraryError,
    refresh: checkAuthStatus,
  } = useGogLibrary();

  useEffect(() => {
    const unlistenSuccess = listen("gog-login-success", () => {
      setIsLoggingIn(false);
      alert.alert({ title: t("Success"), description: t("Successfully logged in to GOG") });
    });

    const unlistenError = listen<string>("gog-login-error", (event) => {
      setIsLoggingIn(false);
      alert.alert({ failure: true, title: t("Login Failed"), description: event.payload });
    });

    return () => {
      unlistenSuccess.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  }, [alert, t]);

  const handleLogin = async () => {
    try {
      setIsLoggingIn(true);
      await invoke("login_to_gog");
    } catch (error) {
      console.error("Failed to initiate GOG login:", error);
      await alert.alert({ failure: true, title: t("Login Failed"), description: String(error) });
      setIsLoggingIn(false);
    }
  };

  const handleLogout = async () => {
    const confirmed = await alert.confirm({
      title: t("Logout from GOG?"),
      description: t("Are you sure you want to logout from GOG?"),
      confirmText: t("Logout"),
      cancelText: t("Cancel"),
    });

    if (!confirmed) {
      return;
    }

    try {
      setIsLoggingOut(true);
      await invoke("logout_from_gog");
      useGogLibrary.getState().clear();
    } catch (error) {
      console.error("Failed to logout from GOG:", error);
      await alert.alert({ failure: true, title: t("Logout Failed"), description: String(error) });
    } finally {
      setIsLoggingOut(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-3">
          <Gamepad2 className="h-10 w-10 object-contain" aria-hidden="true" />
          GOG
        </CardTitle>
        <CardDescription>
          {t("Connect your GOG account to download games from your library")}{" "}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center gap-3">
          {gogAuth.isAuthenticated ? (
            <>
              <CheckCircle className="h-5 w-5 text-green-500" />
              <div className="flex-1">
                <p className="font-medium">{t("Connected to GOG")}</p>
                <p className="text-sm text-muted-foreground">
                  {isLoadingGames
                    ? t("Loading library…")
                    : libraryError
                      ? t("Library could not be loaded")
                      : t("Owned games: {0}", [gogGames.length])}
                </p>
              </div>
              <Badge variant="secondary">{t("Connected")}</Badge>
            </>
          ) : (
            <>
              <XCircle className="h-5 w-5 text-muted-foreground" />
              <div className="flex-1">
                <p className="font-medium">{t("Not Connected")}</p>
                <p className="text-sm text-muted-foreground">
                  {t("Login to access your GOG library")}
                </p>
              </div>
            </>
          )}
        </div>

        <div className="flex flex-wrap gap-2">
          {gogAuth.isAuthenticated ? (
            <Button onClick={handleLogout} variant="outline" disabled={isLoggingOut}>
              <LogOut className="h-4 w-4 mr-2" />
              {isLoggingOut ? t("Logging out...") : t("Logout from GOG")}
            </Button>
          ) : (
            <Button onClick={handleLogin} disabled={isLoggingIn}>
              <LogIn className="h-4 w-4 mr-2" />
              {isLoggingIn ? t("Waiting for login...") : t("Login to GOG")}
            </Button>
          )}
        </div>

        {(gogAuth.isAuthenticated || libraryError) && (
          <ManageMenu label={t("Manage GOG")}>
            {libraryError && (
              <div className="space-y-2">
                <p role="alert" className="text-sm text-destructive">
                  {libraryError}
                </p>
                <BugReportDialog details={libraryError} />
              </div>
            )}
            {gogAuth.isAuthenticated && (
              <Button
                variant="outline"
                className="w-full"
                disabled={isLoadingGames || isLoggingOut}
                onClick={() => void checkAuthStatus()}
              >
                {isLoadingGames ? t("Loading library…") : t("Refresh GOG library")}
              </Button>
            )}
          </ManageMenu>
        )}
      </CardContent>
    </Card>
  );
}
