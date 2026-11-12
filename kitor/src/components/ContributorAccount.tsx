import { UserRound } from "lucide-react";
import { useEffect } from "react";
import { Button } from "@/components/ui/button";
import { useContributor } from "@/store/contributor";
export function GoogleSignIn() {
  const { pending, error, signIn, cancel } = useContributor();
  return (
    <div className="space-y-3">
      <p>
        Sign in with Google to contribute. Your display name and Kitor contributor ID will appear on
        the public pull request. Your email stays private.
      </p>
      {error && (
        <p role="alert" className="text-red-300">
          {error}
        </p>
      )}
      {pending ? (
        <>
          <output>Finish signing in in your browser…</output>
          <Button variant="outline" onClick={() => void cancel()}>
            Cancel sign-in
          </Button>
        </>
      ) : (
        <Button onClick={() => void signIn()}>Sign in with Google</Button>
      )}
    </div>
  );
}
export default function ContributorAccount() {
  const { user, pending, error, refresh, signIn, signOut, cancel } = useContributor();
  useEffect(() => {
    void refresh();
  }, [refresh]);
  return (
    <div className="flex flex-wrap items-center justify-end gap-3 px-6 py-3">
      {user ? (
        <>
          <span className="flex items-center gap-2">
            <UserRound size={18} />
            {user.name}
          </span>
          <Button variant="outline" size="sm" onClick={() => void signOut()}>
            Sign out
          </Button>
        </>
      ) : (
        <Button variant="outline" size="sm" onClick={() => void (pending ? cancel() : signIn())}>
          {pending ? "Cancel sign-in" : "Sign in with Google"}
        </Button>
      )}
      {error && (
        <span role="alert" className="text-sm text-red-300">
          {error}
        </span>
      )}
    </div>
  );
}
