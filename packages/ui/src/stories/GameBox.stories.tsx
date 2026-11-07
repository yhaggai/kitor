import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";
import { GameBox } from "../components/GameBox";
import { games } from "../mocks/games";

const meta = {
  title: "Components/3D game box",
  component: GameBox,
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div style={{ width: "min(380px, 100%)", padding: 24 }}>
        <Story />
      </div>
    ),
  ],
  args: { game: games[0] },
} satisfies Meta<typeof GameBox>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Bilingual: Story = {};
export const MissingArtwork: Story = {
  args: { game: { id: "missing", title: "A new adventure", status: "original" } },
};
export const RotateFlipAndZoom: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(canvas.getByRole("button", { name: "English" }));
    await expect(canvas.getByRole("button", { name: "English" })).toHaveAttribute(
      "aria-pressed",
      "true"
    );
    const target = canvas.getByRole("button", { name: "Rotate Jenny LeClue game box" });
    target.focus();
    await userEvent.keyboard("{ArrowRight}");
    await expect(canvasElement.querySelector("[data-game-cover]")).toHaveStyle({
      "--turn": "-2deg",
    });
    await userEvent.click(canvas.getByRole("button", { name: "Zoom in" }));
    await expect(canvas.getByLabelText("Box zoom level")).toHaveTextContent("125%");
    await userEvent.click(canvas.getByRole("button", { name: "Reset box view" }));
    await expect(canvas.getByLabelText("Box zoom level")).toHaveTextContent("100%");
    await userEvent.click(canvas.getByRole("button", { name: "עברית" }));
  },
};
