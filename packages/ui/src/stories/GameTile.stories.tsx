import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";
import { GameTile } from "../components/GameTile";
import { games } from "../mocks/games";

const meta = {
  title: "Components/Game tile",
  component: GameTile,
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div style={{ width: 230, padding: 24 }}>
        <Story />
      </div>
    ),
  ],
  args: { game: games[0], onOpen: fn() },
} satisfies Meta<typeof GameTile>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Ready: Story = {};
export const TranslationAvailable: Story = { args: { game: games[2] } };
export const NeedsAttention: Story = { args: { game: games[5] } };
export const MissingArtwork: Story = {
  args: { game: { ...games[0], cover: undefined, covers: undefined } },
};
export const HebrewTitle: Story = { args: { game: { ...games[0], title: "ג׳ני לקלו — הבלשית" } } };
export const LongTitle: Story = {
  args: { game: { ...games[1], title: "The Secret of Monkey Island: Special Edition" } },
};
