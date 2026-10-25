import { StrictMode, useState } from "react";
import { createRoot } from "react-dom/client";
import { GameBox, type GameBoxLanguage } from "../src/components/GameBox";
import "../src/index.css";

// Open /examples/game-box.html while the Vite development server is running.
function GameBoxExample() {
  const [language, setLanguage] = useState<GameBoxLanguage>("en");

  return (
    <main className="min-h-screen bg-background px-4 py-8 text-foreground sm:p-8">
      <GameBox
        className="mx-auto max-w-2xl"
        title="The Secret of Monkey Island"
        covers={{
          en: {
            src: "https://assets.kitor.click/covers/monkey-se/en/b9d8b502a92e.jpg",
            title: "The Secret of Monkey Island",
          },
          he: {
            src: "https://assets.kitor.click/covers/monkey-se/he/0808875f8dd2.png",
            title: "הסוד של אי הקופים",
          },
        }}
        language={language}
        onLanguageChange={setLanguage}
      />
    </main>
  );
}

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <GameBoxExample />
    </StrictMode>
  );
}
