// GOG's common.js implements this process/poll flow for external browser login:
// https://static-login.gog-statics.com/bundles/gogloginlogin/js/common.js
// Keep the GOG session in this webview; only the provider step leaves the app.
(() => {
  if (window.location.origin !== "https://login.gog.com") return;

  const providers = new Set(["google", "apple", "steam", "discord", "xbox"]);
  let pending = false;
  let status;
  const showStatus = (message) => {
    if (!status) {
      status = document.createElement("p");
      status.setAttribute("role", "status");
      status.style.cssText = "padding:16px;text-align:center;background:#fff;color:#222";
      document.body.prepend(status);
    }
    status.textContent = message;
  };
  const pause = () => new Promise((resolve) => setTimeout(resolve, 1000));
  const request = async (path, options = {}) => {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 15000);
    try {
      const response = await fetch(path, {
        ...options,
        credentials: "same-origin",
        headers: { "X-Requested-With": "XMLHttpRequest" },
        signal: controller.signal,
      });
      if (!response.ok) throw new Error("GOG request failed");
      return await response.json();
    } finally {
      clearTimeout(timeout);
    }
  };

  // Capture before GOG's popup handler (including clicks on its nested SVG).
  document.addEventListener(
    "click",
    async (event) => {
      const button = event.target.closest?.("a[data-url]");
      if (!button) return;
      let provider;
      try {
        const url = new URL(button.getAttribute("data-url"), window.location.origin);
        const match = url.pathname.match(/^\/login\/providers\/([^/]+)\/init$/);
        if (url.origin !== "https://external-accounts.gog.com" || !match) return;
        provider = match[1];
        if (!providers.has(provider)) return;
      } catch {
        return;
      }
      event.preventDefault();
      event.stopImmediatePropagation();
      if (pending) return;
      pending = true;
      showStatus("Complete sign-in in your browser. Kitor will connect automatically.");
      try {
        const process = await request(`/login_process/${provider}`, { method: "POST" });
        if (typeof process.process_id !== "string" || !process.process_id) {
          throw new Error("Missing login process");
        }
        const destination = new URL(process.process_init_url);
        if (
          destination.origin !== "https://external-accounts.gog.com" ||
          destination.pathname !== `/login/providers/${provider}/init` ||
          destination.username ||
          destination.password
        ) {
          throw new Error("Invalid login destination");
        }
        // Tauri cancels this navigation and opens the URL in the OS browser.
        // This document stays alive, with no popup blocker or window.opener dependency.
        window.location.assign(destination.href);
        const deadline = Date.now() + 240000;
        while (Date.now() < deadline) {
          await pause();
          const result = await request(`/login_process/${encodeURIComponent(process.process_id)}`);
          if (
            result.status === "finished" &&
            typeof result.login_token === "string" &&
            result.login_token
          ) {
            window.location.replace(
              `/on_login_success_external?login_token=${encodeURIComponent(result.login_token)}`
            );
            return;
          }
          if (result.status !== "in_progress") throw new Error("Login did not complete");
        }
        throw new Error("Login timed out");
      } catch {
        // Do not display or log process URLs or tokens.
        showStatus(
          "Sign-in could not be completed. Click the sign-in provider to try again, or close this window."
        );
      } finally {
        pending = false;
      }
    },
    true
  );
})();
