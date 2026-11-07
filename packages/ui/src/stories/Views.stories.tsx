import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";
import {
  emptyHandlers,
  errorHandlers,
  handlers,
  loadingHandlers,
  retryHandlers,
} from "../mocks/handlers";
import { CatalogDemo } from "./CatalogDemo";

const meta = {
  title: "Views/Collections",
  component: CatalogDemo,
  parameters: { msw: { handlers } },
} satisfies Meta<typeof CatalogDemo>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Library: Story = {};
export const Discovery: Story = { args: { initialCollection: "discovery" } };
export const GameDetails: Story = { args: { initialGame: "jenny-leclue" } };
export const DiscoveryDetails: Story = {
  args: { initialCollection: "discovery", initialGame: "grim-fandango-remastered" },
};
export const Loading: Story = { parameters: { msw: { handlers: loadingHandlers } } };
export const Empty: Story = { parameters: { msw: { handlers: emptyHandlers } } };
export const LoadError: Story = { parameters: { msw: { handlers: errorHandlers } } };
export const ZoomAndBack: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const tile = await canvas.findByRole("button", { name: /Explore game Jenny LeClue/ });
    const search = canvas.getByRole("searchbox", { name: "Search games" });
    await userEvent.type(search, "Jenny");
    await userEvent.click(tile);
    await expect(canvas.getByRole("heading", { name: "Jenny LeClue", level: 1 })).toHaveFocus();
    await userEvent.click(canvas.getByRole("tab", { name: "translation" }));
    await expect(canvas.getByRole("heading", { name: "Community translation" })).toBeVisible();
    await userEvent.click(canvas.getByRole("button", { name: "Back to library" }));
    await expect(search).toHaveValue("Jenny");
    await expect(tile).toHaveFocus();
    await expect(
      canvas.queryByRole("button", { name: /Explore game Space Quest V/ })
    ).not.toBeInTheDocument();
  },
};
export const DiscoveryZoomAndBack: Story = {
  args: { initialCollection: "discovery" },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(
      await canvas.findByRole("button", { name: /Explore game Grim Fandango/ })
    );
    await expect(canvas.getByRole("button", { name: "Get the game" })).toBeVisible();
    await userEvent.click(canvas.getByRole("button", { name: "Back to discovery" }));
    await expect(canvas.getByRole("button", { name: /Explore game Grim Fandango/ })).toHaveFocus();
  },
};

export const RetryAfterError: Story = {
  parameters: { msw: { handlers: retryHandlers } },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    await userEvent.click(await canvas.findByRole("button", { name: "Try again" }));
    await expect(
      await canvas.findByRole("button", { name: /Explore game Jenny LeClue/ })
    ).toBeVisible();
    await expect(canvas.queryByRole("alert")).not.toBeInTheDocument();
  },
};

export const InstalledFromDiscovery: Story = {
  args: { initialCollection: "discovery" },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const tile = await canvas.findByRole("button", { name: /Explore game Jenny LeClue/ });
    const search = canvas.getByRole("searchbox", { name: "Search games" });
    await userEvent.type(search, "Jenny");
    await userEvent.click(tile);
    await expect(canvas.getByRole("button", { name: "Play in Hebrew" })).toBeVisible();
    await expect(canvas.queryByRole("button", { name: "View in library" })).not.toBeInTheDocument();
    await userEvent.click(canvas.getByRole("button", { name: "Back to discovery" }));
    await expect(search).toHaveValue("Jenny");
    await expect(tile).toHaveFocus();
  },
};
