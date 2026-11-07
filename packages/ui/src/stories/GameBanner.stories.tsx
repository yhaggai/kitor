import type { Meta, StoryObj } from "@storybook/react-vite";
import { GameBanner } from "../components/GameBanner";
import { games } from "../mocks/games";

const meta = {
  title: "Components/Game banner",
  component: GameBanner,
  args: { game: games[0] },
  tags: ["autodocs"],
} satisfies Meta<typeof GameBanner>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Hebrew: Story = {};
export const English: Story = {
  args: {
    game: {
      ...games[0],
      banner: { en: "/banners/jenny-leclue-en.jpg", logo: "/banners/jenny-leclue-logo-en.png" },
    },
  },
};
export const MissingArtwork: Story = { args: { game: games[2] } };
export const FailedArtwork: Story = {
  args: { game: { ...games[0], banner: { en: "/banners/unavailable.jpg" } } },
};

export const PublishedHebrewOnly: Story = {
  args: {
    game: {
      ...games[0],
      banner: {
        he: "https://assets.kitor.click/banners/jenny-leclue/he/64bf594d7ff5.png",
        aspectRatio: 3,
      },
    },
  },
};
