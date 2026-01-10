import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { DialogProvider } from "./contexts/DialogContext";
import "./index.css";

// Create separate container for dialogs
const dialogRoot = document.createElement("div");
dialogRoot.id = "dialog-root";
document.body.appendChild(dialogRoot);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <DialogProvider dialogRootElement={dialogRoot}>
      <App />
    </DialogProvider>
  </React.StrictMode>
);
