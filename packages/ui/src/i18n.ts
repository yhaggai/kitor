import { useCallback, useSyncExternalStore } from "react";
import hebrew from "./locales/he.json";

export type AppLanguage = "en" | "he";
export const LANGUAGE_STORAGE_KEY = "kitor-language";
const listeners = new Set<() => void>();

export function readLanguage(): AppLanguage {
  try {
    return globalThis.localStorage?.getItem(LANGUAGE_STORAGE_KEY) === "en" ? "en" : "he";
  } catch {
    return "he";
  }
}
let language: AppLanguage = readLanguage();

function applyDirection() {
  if (typeof document === "undefined") return;
  document.documentElement.lang = language;
  document.documentElement.dir = language === "he" ? "rtl" : "ltr";
}
applyDirection();

export function setAppLanguage(value: AppLanguage) {
  language = value === "he" ? "he" : "en";
  try {
    globalThis.localStorage?.setItem(LANGUAGE_STORAGE_KEY, language);
  } catch {
    // The interface remains usable when persistent storage is unavailable.
  }
  applyDirection();
  for (const listener of listeners) listener();
}

export function translate(
  text: string,
  locale: AppLanguage,
  values: readonly (string | number)[] = []
): string {
  const message = locale === "he" ? ((hebrew as Record<string, string>)[text] ?? text) : text;
  return message.replace(/\{(\d+)\}/g, (placeholder, index: string) => {
    const value = values[Number(index)];
    return value === undefined
      ? placeholder
      : locale === "he"
        ? `\u2068${value}\u2069`
        : String(value);
  });
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function useTranslation() {
  const locale = useSyncExternalStore(
    subscribe,
    () => language,
    () => language
  );
  const t = useCallback(
    (text: string, values?: readonly (string | number)[]) => translate(text, locale, values),
    [locale]
  );
  return {
    language: locale,
    setLanguage: setAppLanguage,
    t,
  };
}
