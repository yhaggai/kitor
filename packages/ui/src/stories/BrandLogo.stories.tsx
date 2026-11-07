import type { Meta, StoryObj } from "@storybook/react-vite";
import { BrandLogo } from "../components/BrandLogo";

const meta = {
  title: "Foundations/Brand logo",
  component: BrandLogo,
  tags: ["autodocs"],
  decorators: [
    (Story) => (
      <div style={{ width: 220, padding: 16 }}>
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof BrandLogo>;
export default meta;
export const Sidebar: StoryObj<typeof meta> = {};
