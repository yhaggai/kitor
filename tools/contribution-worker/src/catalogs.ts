import configs from "../../../kitor/contribution-catalogs.json";
import { HttpError } from "./shared";

export type Catalog = {
  id: string;
  title: string;
  label: string;
  path: string;
  header: string[];
  key: number;
  english?: number;
  editable: number[];
  status?: number;
  locked_keys?: string[];
  skip: number;
  tokens: string;
};

export function catalog(id: string = "drifter"): Catalog {
  const result = configs.find((entry) => entry.id === id);
  if (!result) throw new HttpError(400, "Unknown translation catalog");
  return result;
}

export const catalogIds = configs.map(({ id }) => id);

export const MAX_CATALOG_BYTES = 24_000_000;
