// The shared box uses the page's explicit locale on the website.
export function useTranslation() {
  return {
    t: (message: string, values: readonly (string | number)[] = []) =>
      message.replace(/\{(\d+)\}/g, (placeholder, index: string) =>
        String(values[Number(index)] ?? placeholder),
      ),
  };
}
