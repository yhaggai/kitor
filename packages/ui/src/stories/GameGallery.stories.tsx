import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";
import galleries from "../../../../kitor/src/data/game-galleries.json";
import { GameGallery } from "../components/GameGallery";

const meta = {
  title: "Components/Game gallery",
  component: GameGallery,
  args: { title: "Space Quest I", screenshots: galleries.sq1sci },
} satisfies Meta<typeof GameGallery>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Screenshots: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(canvas.getByRole("button", { name: "Show screenshot 3" }));
    await expect(canvas.getByRole("button", { name: "Show screenshot 3" })).toHaveAttribute(
      "aria-pressed",
      "true"
    );
    await userEvent.click(canvas.getByRole("button", { name: "Enlarge screenshot 3" }));
    const dialog = within(canvas.getByRole("dialog"));
    await userEvent.click(dialog.getByRole("button", { name: "Next screenshot" }));
    await expect(dialog.getByText("4 / 5")).toBeVisible();
    await userEvent.keyboard("{ArrowLeft}");
    await expect(dialog.getByText("3 / 5")).toBeVisible();
    await userEvent.click(dialog.getByRole("button", { name: "Close" }));
    await expect(canvas.queryByRole("dialog")).not.toBeInTheDocument();
    await expect(canvas.getByRole("button", { name: "Enlarge screenshot 3" })).toHaveFocus();
  },
};
export const SingleImage: Story = { args: { screenshots: galleries.sq1sci.slice(0, 1) } };
export const NoImages: Story = { args: { screenshots: [] } };
export const BrokenImage: Story = { args: { screenshots: [{ src: "/missing-screenshot.jpg" }] } };
