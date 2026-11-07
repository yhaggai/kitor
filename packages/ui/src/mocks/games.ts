import type { GameSummary } from "../types";

/** Deliberately fictional installation state; no native APIs or user library reads. */
const fixtures: GameSummary[] = [
  {
    id: "jenny-leclue",
    banner: {
      en: "/banners/jenny-leclue-en.jpg",
      he: "/banners/jenny-leclue-he.png",
      logo: "/banners/jenny-leclue-logo-en.png",
    },
    title: "Jenny LeClue",
    cover: "/covers/jenny-leclue-he.jpg",
    status: "ready",
    language: "Hebrew",
    author: "Example community team",
    version: "1.0",
    description:
      "A small-town mystery. A sharp young detective. An adventure full of choices.\n\nStep into the shoes of Jenny LeClue and uncover the secrets of Arthurton.",
  },
  {
    id: "monkey-se",
    banner: {
      en: "/banners/monkey-classic-en.png",
      he: "/banners/monkey-classic-he.png",
      aspectRatio: 16 / 9,
    },
    title: "The Secret of Monkey Island — Classic",
    cover: "/covers/monkey-classic-en.jpg",
    status: "ready",
    language: "Hebrew",
    description: "Set sail for a witty pirate adventure full of puzzles and unexpected encounters.",
  },
  {
    id: "tentacle",
    title: "Day of the Tentacle",
    cover: "/covers/tentacle.jpg",
    status: "available",
    language: "Hebrew",
    description: "Three friends, three eras, and one very ambitious purple tentacle.",
  },
  {
    id: "grim-fandango-remastered",
    title: "Grim Fandango",
    cover: "/covers/grim-fandango-remastered.jpg",
    status: "available",
    language: "Hebrew",
    description: "A noir journey through the Land of the Dead.",
  },
  {
    id: "sq5",
    title: "Space Quest V",
    cover: "/covers/sq5.jpg",
    status: "ready",
    language: "Hebrew",
    description: "Roger Wilco sets off on another unlikely adventure among the stars.",
  },
  {
    id: "drifter",
    title: "The Drifter",
    cover: "/covers/drifter.jpg",
    status: "attention",
    language: "Hebrew",
    description:
      "A pulpy thriller about a man caught in a conspiracy far beyond his understanding.",
  },
  {
    id: "monkey2-se",
    title: "Monkey Island 2: LeChuck's Revenge — Classic",
    cover: "/covers/monkey2-classic-en.jpg",
    banner: {
      en: "/banners/monkey2-classic-en.png",
      he: "/banners/monkey2-classic-he.png",
      aspectRatio: 16 / 9,
    },
    status: "ready",
    language: "Hebrew",
    description: "Guybrush returns for a treasure hunt and another encounter with LeChuck.",
  },
];

export const games: GameSummary[] = fixtures.map((game) => ({
  ...game,
  covers: {
    he: { src: game.cover ?? "" },
    en: {
      src:
        game.id === "monkey-se" || game.id === "monkey2-se"
          ? (game.cover ?? "")
          : `/covers/${game.id}-en.jpg`,
    },
  },
}));
