import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";
import { Button } from "../components/Button";
import { games } from "../mocks/games";
import { GameDetailsView } from "../views/GameDetailsView";

const meta = {
  title: "Views/Game details",
  component: GameDetailsView,
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div style={{ padding: 32 }}>
        <Story />
      </div>
    ),
  ],
  args: {
    game: games[0],
    collection: "library",
    onBack: fn(),
    actions: <Button onClick={fn()}>Play in Hebrew</Button>,
  },
} satisfies Meta<typeof GameDetailsView>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Installed: Story = {};
export const TranslationAvailable: Story = {
  args: { game: games[2], actions: <Button onClick={fn()}>Install translation</Button> },
};
export const Unowned: Story = {
  args: {
    collection: "discovery",
    game: { ...games[3], status: "unowned" },
    actions: <Button onClick={fn()}>Get the game</Button>,
  },
};
export const MissingMetadata: Story = {
  args: {
    game: { id: "unknown", title: "An undiscovered adventure", status: "original" },
    actions: undefined,
  },
};
export const HebrewContent: Story = {
  args: {
    game: {
      ...games[0],
      title: "ג׳ני לקלו — הבלשית",
      description:
        "תעלומה בעיירה קטנה, בלשית צעירה והרפתקה מלאה בבחירות.\n\nגלו את הסודות שמסתתרים בעיירה.",
    },
  },
};
