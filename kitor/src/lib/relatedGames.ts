type Candidate = {
  id: string;
  title: string;
  tags?: string[];
  addedOn?: string;
};

function titleWords(title: string) {
  return new Set(
    title
      .toLowerCase()
      .match(/[a-z0-9]+/g)
      ?.filter(
        (word) =>
          word.length > 2 &&
          !["the", "and", "for", "edition", "special", "remastered"].includes(word)
      )
  );
}

export function relatedGames<T extends Candidate>(current: T, catalog: T[]): T[] {
  const words = titleWords(current.title);
  const tags = new Set(current.tags?.filter((tag) => tag !== "hot-kitor"));
  return catalog
    .filter((game) => game.id !== current.id)
    .map((game) => {
      const titleMatches = [...titleWords(game.title)].filter((word) => words.has(word)).length;
      const tagMatches = game.tags?.filter((tag) => tags.has(tag)).length ?? 0;
      return { game, score: titleMatches * 2 + tagMatches };
    })
    .sort(
      (a, b) =>
        b.score - a.score ||
        (b.game.addedOn ?? "").localeCompare(a.game.addedOn ?? "") ||
        a.game.id.localeCompare(b.game.id)
    )
    .slice(0, 3)
    .map(({ game }) => game);
}
