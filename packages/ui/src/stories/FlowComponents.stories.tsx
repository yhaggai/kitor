import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";
import { expect, fn, userEvent, within } from "storybook/test";
import { ActionCard, Button, FlowDialog, ManageMenu, OperationProgress } from "../index";

function ComponentsDemo() {
  const [open, setOpen] = useState(false);
  return (
    <div style={{ padding: 40, maxWidth: 600 }}>
      <ActionCard
        title="Game management"
        description="Secondary actions live behind the gear button."
      >
        <div className="kt-flow-actions">
          <Button onClick={fn()}>Play in Hebrew</Button>
          <ManageMenu>
            <div className="kt-flow-actions">
              <Button variant="secondary" onClick={fn()}>
                Open game folder
              </Button>
              <Button variant="secondary" onClick={fn()}>
                Refresh game
              </Button>
              <Button variant="secondary" onClick={() => setOpen(true)}>
                Restore original game
              </Button>
            </div>
          </ManageMenu>
        </div>
      </ActionCard>
      {open && (
        <FlowDialog
          title="Restore original game?"
          description="Remove the translation and restore original files from backup."
          onClose={() => setOpen(false)}
          footer={<Button onClick={() => setOpen(false)}>Restore</Button>}
        />
      )}
    </div>
  );
}
const meta = { title: "Components/Management controls", component: ComponentsDemo } satisfies Meta<
  typeof ComponentsDemo
>;
export default meta;
type Story = StoryObj<typeof meta>;
export const GearDropdown: Story = {};
export const Progress: Story = {
  render: () => (
    <div style={{ padding: 40 }}>
      <OperationProgress value={65} step="Installing translation" />
    </div>
  ),
};
export const KeyboardMenu: Story = {
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    const gear = c.getByRole("button", { name: "Manage game" });
    await userEvent.click(gear);
    await expect(c.getByRole("button", { name: "Open game folder" })).toHaveFocus();
    await userEvent.keyboard("{ArrowDown}");
    await expect(c.getByRole("button", { name: "Refresh game" })).toHaveFocus();
    await userEvent.keyboard("{Escape}");
    await expect(gear).toHaveFocus();
    await expect(gear).toHaveAttribute("aria-expanded", "false");
  },
};
