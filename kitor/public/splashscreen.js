(() => {
  const scenes = [
    "/splash-scenes/israeli-wedding-branded.png",
    "/splash-scenes/tel-aviv-beach-branded.png",
    "/splash-scenes/passover-seder-branded.png",
    "/splash-scenes/spring-picnic-branded.png",
  ];
  const storageKey = "kitor.lastSplashScene";
  let previous;
  try {
    previous = localStorage.getItem(storageKey);
  } catch {
    // The splash still works when storage is unavailable.
  }
  const choices = scenes.filter((scene) => scene !== previous);
  const selected = choices[Math.floor(Math.random() * choices.length)];
  const logo = document.getElementById("logo");
  // Keep the live indicator in the clear corner of each recomposed scene.
  const indicatorPositions = {
    "/splash-scenes/israeli-wedding-branded.png": ["95.7%", "94.4%"],
    "/splash-scenes/tel-aviv-beach-branded.png": ["97.2%", "95.8%"],
  };
  const [left, top] = indicatorPositions[selected] ?? ["96%", "95%"];
  logo.parentElement.style.setProperty("--loader-left", left);
  logo.parentElement.style.setProperty("--loader-top", top);

  logo.addEventListener("load", () => {
    if (logo.getAttribute("src") !== selected) return;
    try {
      localStorage.setItem(storageKey, selected);
    } catch {
      // Remembering the last scene is optional.
    }
  });
  logo.addEventListener(
    "error",
    () => {
      logo.src = "/splash-image.webp";
    },
    { once: true }
  );
  logo.src = selected;
})();
