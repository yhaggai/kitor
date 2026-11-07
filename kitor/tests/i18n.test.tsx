import { afterEach, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { AppShell } from "../../packages/ui/src/components/AppShell";
import { readLanguage, setAppLanguage, translate } from "../../packages/ui/src/i18n";
import hebrew from "../../packages/ui/src/locales/he.json";
import { CollectionView } from "../../packages/ui/src/views/CollectionView";
import { GameDetailsView } from "../../packages/ui/src/views/GameDetailsView";

afterEach(() => setAppLanguage("en"));

test("Hebrew and English render through the same navigation and collection components", () => {
  const render = () =>
    renderToStaticMarkup(
      <AppShell active="library" onNavigate={() => {}}>
        <CollectionView collection="library" games={[]} onOpen={() => {}} />
      </AppShell>
    );
  expect(render()).toContain("Your library");
  setAppLanguage("he");
  expect(render()).toContain("הספרייה שלכם");
  expect(render()).toContain("גילוי תרגומים");
  setAppLanguage("en");
  expect(render()).toContain("Your library");
});

test("translations preserve interpolation slots and isolate mixed-direction values", () => {
  for (const [english, translation] of Object.entries(hebrew)) {
    expect(translation.trim().length).toBeGreaterThan(0);
    expect(translation.match(/\{\d+\}/g)?.sort() ?? []).toEqual(
      english.match(/\{\d+\}/g)?.sort() ?? []
    );
  }
  expect(translate("Version {0}", "he", ["1.2.3"])).toBe("גרסה \u20681.2.3\u2069");
  expect(translate("Unknown external error", "he")).toBe("Unknown external error");
});

test("language persists and invalid or inaccessible storage falls back to English", () => {
  const original = Object.getOwnPropertyDescriptor(globalThis, "localStorage");
  let stored: string | null = null;
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: {
      getItem: () => stored,
      setItem: (_key: string, value: string) => {
        stored = value;
      },
    },
  });
  try {
    setAppLanguage("he");
    expect(readLanguage()).toBe("he");
    setAppLanguage("en");
    expect(readLanguage()).toBe("en");
    stored = "invalid";
    expect(readLanguage()).toBe("en");
    Object.defineProperty(globalThis, "localStorage", {
      configurable: true,
      get() {
        throw new Error("Unavailable");
      },
    });
    expect(readLanguage()).toBe("en");
    expect(() => setAppLanguage("he")).not.toThrow();
  } finally {
    if (original) Object.defineProperty(globalThis, "localStorage", original);
    else Reflect.deleteProperty(globalThis, "localStorage");
  }
});

test("translated tabs retain stable IDs and accessible panel relationships", () => {
  setAppLanguage("he");
  const html = renderToStaticMarkup(
    <GameDetailsView
      game={{ id: "test", title: "Test", status: "ready" }}
      collection="library"
      onBack={() => {}}
    />
  );
  expect(html).toContain("סקירה");
  expect(html).toContain('-overview"');
  expect(html).not.toContain('-סקירה"');
  expect(html).toContain("התרגום מותקן");
});
