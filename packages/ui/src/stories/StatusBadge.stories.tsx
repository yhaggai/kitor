import type { Meta, StoryObj } from "@storybook/react-vite";
import { StatusBadge } from "../components/StatusBadge";

const meta = {
  title: "Components/Status badge",
  component: StatusBadge,
  tags: ["autodocs"],
  parameters: { layout: "centered" },
  args: { status: "ready" },
} satisfies Meta<typeof StatusBadge>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Ready: Story = {};
export const Available: Story = { args: { status: "available" } };
export const Unowned: Story = { args: { status: "unowned" } };
export const Attention: Story = { args: { status: "attention" } };
export const Original: Story = { args: { status: "original" } };
