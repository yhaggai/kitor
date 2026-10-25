import { Package } from "lucide-react";
import { useState } from "react";
import placeholders from "@/data/cover-placeholders.json";
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

function ProgressiveCover({ gameId, coverUrl, title }: CoverImageProps) {
  const [loaded, setLoaded] = useState(false);
  const [failed, setFailed] = useState(false);
  const src = getCoverSrc(coverUrl);
  const placeholder = (placeholders as Record<string, string>)[gameId];

  return (
    <div className="relative w-full h-full overflow-hidden rounded-t-lg flex items-center justify-center">
      {placeholder ? (
        <img
          src={placeholder}
          alt=""
          aria-hidden="true"
          className="absolute inset-0 w-full h-full object-cover blur-sm scale-105"
        />
      ) : <Package aria-hidden="true" className="h-16 w-16 text-muted-foreground" />}
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
      ) : <span className="sr-only">{title}</span>}
    </div>
  );
}
