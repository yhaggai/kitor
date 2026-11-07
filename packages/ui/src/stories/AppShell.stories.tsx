import type { Meta, StoryObj } from "@storybook/react-vite";
import { fn } from "storybook/test";
import { AppShell } from "../components/AppShell";

const meta = {
  title: "Components/App shell",
  component: AppShell,
  tags: ["autodocs"],
  args: {
    active: "library",
    onNavigate: fn(),
    children: (
      <>
        <h1>Your library</h1>
        <p>The shell provides navigation and the shared content area.</p>
      </>
    ),
  },
} satisfies Meta<typeof AppShell>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Library: Story = {};
export const Discovery: Story = { args: { active: "discovery" } };
export const Settings: Story = { args: { active: "settings" } };
