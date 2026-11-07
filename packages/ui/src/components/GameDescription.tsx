import { ChevronDown, ChevronUp } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "../i18n";
import { Button } from "./Button";

export function GameDescription({ description }: { description: string }) {
  const { t } = useTranslation();
  const id = useId();
  const text = useRef<HTMLParagraphElement>(null);
  const [expanded, setExpanded] = useState(false);
  const [overflowing, setOverflowing] = useState(false);

  useEffect(() => {
    const element = text.current;
    if (!element || expanded) return;
    const measure = () => setOverflowing(element.scrollHeight > element.clientHeight + 1);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, [expanded]);

  return (
    <>
      <p ref={text} id={id} className="kt-detail-intro" data-expanded={expanded} dir="auto">
        {description}
      </p>
      {(overflowing || expanded) && (
        <Button
          variant="ghost"
          aria-expanded={expanded}
          aria-controls={id}
          onClick={() => setExpanded(!expanded)}
        >
          {t(expanded ? "Show less" : "Read more")}
          {expanded ? (
            <ChevronUp size={16} aria-hidden="true" />
          ) : (
            <ChevronDown size={16} aria-hidden="true" />
          )}
        </Button>
      )}
    </>
  );
}
