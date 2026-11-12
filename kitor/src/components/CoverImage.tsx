import { Package } from "lucide-react";
import { useState } from "react";
import { getCoverSrc } from "@/lib/utils";

type CoverImageProps = {
  gameId: string;
  coverUrl?: string | null;
  title: string;
};

export function CoverImage(props: CoverImageProps) {
  // A changed source gets its own loading state, including cached-file replacements.
  return <ProgressiveCover key={`${props.gameId}:${props.coverUrl}`} {...props} />;
}

function ProgressiveCover({ coverUrl, title }: CoverImageProps) {
  const [loaded, setLoaded] = useState(false);
  const [failed, setFailed] = useState(false);
  const src = getCoverSrc(coverUrl);

  return (
    <div className="relative w-full h-full overflow-hidden rounded-t-lg flex items-center justify-center">
      <div
        className="flex flex-col items-center gap-2 p-3 text-center text-muted-foreground"
        aria-hidden={!!src && loaded && !failed}
      >
        <Package aria-hidden="true" className="h-12 w-12" />
        <span dir="auto">{title}</span>
      </div>
      {src && !failed ? (
        <img
          src={src}
          alt={title}
          loading="lazy"
          decoding="async"
          onLoad={() => setLoaded(true)}
          onError={() => setFailed(true)}
          className={`absolute inset-0 w-full h-full object-cover transition-opacity duration-300 motion-reduce:transition-none ${loaded ? "opacity-100" : "opacity-0"}`}
        />
      ) : null}
    </div>
  );
}
