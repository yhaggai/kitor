// GameBox supplies its own Hebrew labels when the site selects Hebrew.
export function useTranslation() {
  return {
    t(message: string, values: readonly string[] = []) {
      return message.replace(/\{(\d+)\}/g, (token, index) => values[Number(index)] ?? token);
    },
  };
}
