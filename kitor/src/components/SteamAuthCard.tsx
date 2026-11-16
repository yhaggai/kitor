import { steamIcon, useTranslation } from "@kitor/ui";
import NativeSteamConnection from "@/components/NativeSteamConnection";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export default function SteamAuthCard() {
  const { t } = useTranslation();
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-3">
          <img className="h-10 w-10 object-contain" src={steamIcon} alt="" />
          Steam
        </CardTitle>
        <CardDescription>
          {t("Connect Steam to find games you own and download supported games directly in Kitor.")}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <NativeSteamConnection />
      </CardContent>
    </Card>
  );
}
