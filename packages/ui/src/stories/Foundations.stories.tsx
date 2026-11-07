import type { Meta, StoryObj } from "@storybook/react-vite";

function Foundations() {
  return (
    <div className="kt-foundations">
      <p className="kt-eyebrow">KITOR DESIGN SYSTEM</p>
      <h1>Warm collection</h1>
      <p>Cream and navy branding. Charcoal shelves. Golden-orange actions.</p>
      <div className="kt-swatches">
        {[
          "brand-cream",
          "brand-navy",
          "brand-orange",
          "brand-gold",
          "bg",
          "sidebar",
          "surface",
          "text",
          "muted",
          "border",
          "accent",
          "success",
          "danger",
        ].map((token) => (
          <div className="kt-swatch" key={token}>
            <span style={{ background: `var(--kt-${token})` }} />
            {token}
            <br />
            <code>--kt-{token}</code>
          </div>
        ))}
      </div>
      <h2>Typography</h2>
      <p style={{ fontSize: 40 }}>Your next adventure</p>
      <p style={{ fontSize: 24 }} dir="rtl">
        ההרפתקה הבאה שלכם
      </p>
      <p>Inter / Noto Sans Hebrew / system sans. Body 15px. Headings 28–56px.</p>
      <h2 style={{ marginTop: 32 }}>Spacing & shape</h2>
      <p>4 · 8 · 12 · 16 · 24 · 32 · 48px. Controls 8px radius. Surfaces 10px.</p>
      <h2 style={{ marginTop: 32 }}>Motion & accessibility</h2>
      <p>
        Cover zoom: 380ms, cubic-bezier(.22, 1, .36, 1). Reduced motion skips the zoom. Detail
        headings receive focus; Back restores focus, search, and scroll.
      </p>
    </div>
  );
}
export default { title: "Foundations/Design system", component: Foundations } satisfies Meta<
  typeof Foundations
>;
export const Tokens: StoryObj<typeof Foundations> = {};
