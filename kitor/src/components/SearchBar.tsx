import { Search } from "lucide-react";

type SearchBarProps = {
  label: string;
  value: string;
  onChange: (value: string) => void;
};

export function SearchBar({ label, value, onChange }: SearchBarProps) {
  return (
    <div className="relative w-full sm:max-w-md">
      <Search
        aria-hidden="true"
        className="pointer-events-none absolute left-3 top-3 h-4 w-4 text-muted-foreground"
      />
      <input
        type="search"
        aria-label={label}
        placeholder={`${label}…`}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Escape") onChange("");
        }}
        className="h-10 w-full rounded-md border border-input bg-background pl-9 pr-3 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
      />
    </div>
  );
}
