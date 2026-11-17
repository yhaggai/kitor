import { useTranslation } from "@kitor/ui";
import { PencilLine } from "lucide-react";
import { lazy, Suspense, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { useContributor } from "@/store/contributor";
import catalogs from "../../contribution-catalogs.json";
import { GoogleSignIn } from "./ContributorAccount";

export type ContributionCatalog = (typeof catalogs)[number];
const Editor = lazy(() => import("./TranslationEditor"));

export function TranslationContributionButton({ gameId }: { gameId: string }) {
  const { t, language } = useTranslation();
  const [open, setOpen] = useState(false);
  const [catalogId, setCatalogId] = useState<string | null>(null);
  const user = useContributor((s) => s.user);
  const options = catalogs.filter((catalog) => catalog.game === gameId);
  if (!options.length) return null;
  const catalog =
    options.find((item) => item.id === catalogId) ?? (options.length === 1 ? options[0] : null);
  const close = () => {
    setCatalogId(null);
    setOpen(false);
  };
  return (
    <>
      <Button variant="outline" className="w-full" onClick={() => setOpen(true)}>
        <PencilLine className="mr-2 h-4 w-4" />
        {t("Improve translation")}
      </Button>
      {open && !user && (
        <Dialog open onOpenChange={(next) => !next && close()}>
          <DialogContent dir={language === "he" ? "rtl" : "ltr"}>
            <DialogTitle>{t("Sign in to improve this translation")}</DialogTitle>
            <DialogDescription>
              {t("Continue with your Google account, then edit and review your corrections.")}
            </DialogDescription>
            <GoogleSignIn />
          </DialogContent>
        </Dialog>
      )}
      {open && user && !catalog && (
        <Dialog open onOpenChange={(next) => !next && close()}>
          <DialogContent dir={language === "he" ? "rtl" : "ltr"}>
            <DialogTitle>{t("Choose a translation")}</DialogTitle>
            <DialogDescription>
              {t("Which Hebrew edition would you like to edit?")}
            </DialogDescription>
            {options.map(({ id, label }) => (
              <Button key={id} variant="outline" onClick={() => setCatalogId(id)}>
                {t(label)}
              </Button>
            ))}
          </DialogContent>
        </Dialog>
      )}
      {open && user && catalog && (
        <Suspense fallback={<output>{t("Opening translation editor…")}</output>}>
          <Editor
            key={user.id + ":" + catalog.id}
            catalog={catalog}
            contributor={user}
            onClose={close}
          />
        </Suspense>
      )}
    </>
  );
}
