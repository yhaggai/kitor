import type { Meta, StoryObj } from "@storybook/react-vite";
import banners from "../../../../kitor/src/data/game-banners.json";
import covers from "../../../../kitor/src/data/game-box-covers.json";
import { GameTile } from "../components/GameTile";
import type { GameSummary } from "../types";
import { GameDetailsView } from "../views/GameDetailsView";

const game: GameSummary = {
  id: "disco-elysium",
  title: "דיסקו אליסיום",
  status: "original",
  covers: covers["disco-elysium"],
  banner: banners["disco-elysium"],
  cover: covers["disco-elysium"].en.src,
  description:
    "תרגום עברי חלקי ל־Disco Elysium – The Final Cut. חבילת בדיקה מקומית; הורדה ציבורית עדיין אינה זמינה.",
};
const meta = {
  title: "Games/Disco Elysium",
  component: GameDetailsView,
  args: { game, collection: "library", onBack: () => {}, actions: null },
} satisfies Meta<typeof GameDetailsView>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Original: Story = {};
export const HebrewPreview: Story = {
  args: {
    game: {
      ...game,
      status: "ready",
      cover: covers["disco-elysium"].he.src,
      language: "Hebrew",
      version: "0.1.0-preview1",
    },
  },
};
export const Uninstalled: Story = {};
export const ImageFailure: Story = {
  args: {
    game: {
      ...game,
      cover: "/missing-disco-cover.png",
      covers: { en: { src: "/missing-disco-cover.png" }, he: { src: "/missing-disco-he.png" } },
      banner: { en: "/missing-disco-banner.png", aspectRatio: 2.14 },
    },
  },
};

export const LibraryCards: Story = {
  render: () => (
    <div className="kt-game-grid" style={{ maxWidth: 480, padding: 24 }}>
      <GameTile game={game} onOpen={() => {}} />
      <GameTile
        game={{ ...game, status: "ready", cover: covers["disco-elysium"].he.src }}
        onOpen={() => {}}
      />
    </div>
  ),
};
