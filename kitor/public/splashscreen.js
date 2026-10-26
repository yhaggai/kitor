(() => {
  const scenes = [
    '/splash-scenes/israeli-wedding.webp',
    '/splash-scenes/tel-aviv-beach.webp',
    '/splash-scenes/passover-seder.webp',
    '/splash-scenes/spring-picnic.webp',
  ];
  const storageKey = 'kitor.lastSplashScene';
  let previous;
  try {
    previous = localStorage.getItem(storageKey);
  } catch {
    // The splash still works when storage is unavailable.
  }
  const choices = scenes.filter((scene) => scene !== previous);
  const selected = choices[Math.floor(Math.random() * choices.length)];
  const logo = document.getElementById('logo');
  logo.addEventListener('load', () => {
    if (logo.getAttribute('src') !== selected) return;
    try {
      localStorage.setItem(storageKey, selected);
    } catch {
      // Remembering the last scene is optional.
    }
  });
  logo.addEventListener('error', () => {
    logo.src = '/splash-image.webp';
  }, { once: true });
  logo.src = selected;
})();
