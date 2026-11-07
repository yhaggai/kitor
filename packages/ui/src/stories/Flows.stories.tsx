import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect, userEvent, within } from "storybook/test";
import { handlers } from "../mocks/handlers";
import { CatalogDemo } from "./CatalogDemo";

const meta = {
  title: "Flows/Desktop features",
  component: CatalogDemo,
  parameters: { msw: { handlers } },
} satisfies Meta<typeof CatalogDemo>;
export default meta;
type Story = StoryObj<typeof meta>;
export const FullApp: Story = {};
export const Settings: Story = { args: { initialSettings: true } };
export const DisconnectedAccounts: Story = { args: { initialSettings: true, disconnected: true } };
export const TranslationUpdate: Story = {
  args: { initialGame: "jenny-leclue", initialFlow: "update" },
};
export const RestoreConfirmation: Story = {
  args: { initialGame: "jenny-leclue", initialFlow: "restore" },
};
export const RemoveConfirmation: Story = {
  args: { initialGame: "jenny-leclue", initialFlow: "remove" },
};
export const ClearLibrary: Story = { args: { initialFlow: "remove-all" } };
export const FolderDetection: Story = { args: { initialFlow: "folder" } };
export const UnrecognizedFolder: Story = { args: { initialFlow: "folder", scenario: "no-match" } };
export const ScanWarnings: Story = { args: { initialFlow: "scan", scenario: "scan-warnings" } };
export const SteamDownload: Story = {
  args: {
    initialCollection: "discovery",
    initialGame: "grim-fandango-remastered",
    initialFlow: "steam",
  },
};
export const GogDownload: Story = {
  args: {
    initialCollection: "discovery",
    initialGame: "grim-fandango-remastered",
    initialFlow: "gog",
  },
};
export const GogSignInRequired: Story = { args: { ...GogDownload.args, disconnected: true } };
export const UnsupportedEdition: Story = {
  args: { ...SteamDownload.args, scenario: "unsupported" },
};
export const MissingGameFiles: Story = { args: { initialGame: "drifter" } };
export const RuntimeRequired: Story = { args: { initialGame: "sq5", scenario: "missing-runtime" } };
export const SteamPrivateLibrary: Story = {
  args: { initialSettings: true, initialFlow: "refresh-steam", scenario: "private-library" },
};
export const AppUpdate: Story = { args: { initialSettings: true, initialFlow: "check-app" } };
export const AppUpToDate: Story = {
  args: { initialSettings: true, initialFlow: "check-app", scenario: "up-to-date" },
};
export const BugReport: Story = { args: { initialFlow: "report" } };

export const InstallRestoreAndRemove: Story = {
  args: { initialGame: "tentacle" },
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await userEvent.click(await c.findByRole("button", { name: "Install translation" }));
    await userEvent.click(c.getByRole("button", { name: "Continue" }));
    await expect(await c.findByRole("progressbar")).toBeVisible();
    await c.findByRole("heading", { name: "Complete" }, { timeout: 6000 });
    await userEvent.click(c.getByRole("button", { name: "Close" }));
    await expect(c.getByRole("button", { name: "Play in Hebrew" })).toBeVisible();
    await userEvent.click(c.getByRole("button", { name: "Manage game" }));
    await userEvent.click(c.getByRole("button", { name: "Restore original game" }));
    await userEvent.click(c.getByRole("button", { name: "Confirm" }));
    await c.findByRole("heading", { name: "Complete" }, { timeout: 6000 });
    await userEvent.click(c.getByRole("button", { name: "Close" }));
    await expect(c.getByRole("button", { name: "Play original" })).toBeVisible();
    await userEvent.click(c.getByRole("button", { name: "Manage game" }));
    await userEvent.click(c.getByRole("button", { name: "Remove game" }));
    await expect(c.getByText(/Game files on disk will not be deleted/)).toBeVisible();
    await userEvent.click(c.getByRole("button", { name: "Confirm" }));
    await c.findByRole("heading", { name: "Complete" }, { timeout: 6000 });
    await userEvent.click(c.getByRole("button", { name: "Close" }));
    await expect(
      c.queryByRole("button", { name: /Explore Day of the Tentacle/ })
    ).not.toBeInTheDocument();
  },
};
export const FailedInstallRetry: Story = {
  args: { initialGame: "tentacle", initialFlow: "install", scenario: "failure" },
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await userEvent.click(await c.findByRole("button", { name: "Continue" }));
    await expect(await c.findByRole("alert", {}, { timeout: 6000 })).toHaveTextContent(
      "could not finish"
    );
    await userEvent.click(c.getByRole("button", { name: "Try again" }));
    await c.findByRole("heading", { name: "Complete" }, { timeout: 6000 });
    await userEvent.click(c.getByRole("button", { name: "Close" }));
    await expect(c.getByRole("button", { name: "Play in Hebrew" })).toBeVisible();
  },
};
export const DownloadHideAndComplete: Story = {
  args: GogDownload.args,
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await expect(await c.findByRole("button", { name: "Continue" })).toBeDisabled();
    await userEvent.click(c.getByRole("button", { name: "Choose folder" }));
    await userEvent.click(c.getByRole("button", { name: "Continue" }));
    await userEvent.click(await c.findByRole("button", { name: "Hide download" }));
    await userEvent.click(c.getByRole("button", { name: "Show progress" }));
    await c.findByRole("heading", { name: "Complete" }, { timeout: 6000 });
    await userEvent.click(c.getByRole("button", { name: "Close" }));
    await userEvent.click(c.getByRole("button", { name: "View in library" }));
    await expect(await c.findByRole("button", { name: "Play original" })).toBeVisible();
    await expect(c.getByRole("button", { name: "Install translation" })).toBeVisible();
  },
};

export const AccountsRuntimeAndUpdate: Story = {
  args: { initialSettings: true, disconnected: true, scenario: "missing-runtime" },
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    async function complete(start: string, confirm = "Continue") {
      await userEvent.click(await c.findByRole("button", { name: start }));
      await userEvent.click(c.getByRole("button", { name: confirm }));
      await c.findByRole("heading", { name: "Complete" }, { timeout: 6000 });
      await userEvent.click(c.getByRole("button", { name: "Close" }));
    }
    await complete("Connect GOG", "Continue to sign in");
    await expect(c.getByRole("button", { name: "Log out of GOG" })).toBeVisible();
    await complete("Download ScummVM");
    await expect(c.getByRole("button", { name: "Open ScummVM" })).toBeVisible();
    await complete("Check for app updates");
    await complete("Update and restart");
    await complete("Restart Kitor");
    await expect(c.getByText("Kitor is up to date.")).toBeVisible();
  },
};
export const HebrewBanner: Story = { args: { initialGame: "jenny-leclue" } };
