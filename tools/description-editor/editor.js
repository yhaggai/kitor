const search = document.querySelector("#search");
const list = document.querySelector("#games");
const title = document.querySelector("#title");
const gameId = document.querySelector("#game-id");
const form = document.querySelector("#editor");
const description = document.querySelector("#description");
const status = document.querySelector("#status");
const reset = document.querySelector("#reset");
const submit = document.querySelector("#submit");

let games = [];
let sha;
let selected;
let submitting = false;
const drafts = new Map();
const submitted = new Map();

function syncButtons() {
  const changed = selected && description.value !== selected.description;
  reset.disabled = submitting || !changed;
  submit.disabled =
    submitting ||
    !changed ||
    !description.value.trim() ||
    submitted.get(selected.id) === description.value;
  description.readOnly = submitting;
}

function renderGames() {
  const query = search.value.trim().toLowerCase();
  const visible = games.filter((game) =>
    `${game.title} ${game.englishTitle} ${game.id}`.toLowerCase().includes(query)
  );
  list.replaceChildren();
  if (visible.length === 0) {
    list.textContent = "No matching games";
    return;
  }

  for (const game of visible) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = game.title;
    button.dir = "auto";
    button.title = game.id;
    if (game === selected) button.setAttribute("aria-current", "true");
    button.addEventListener("click", () => {
      selected = game;
      title.textContent = game.title;
      title.dir = "auto";
      gameId.textContent =
        game.englishTitle && game.englishTitle !== game.title
          ? `${game.englishTitle} · ${game.id}`
          : game.id;
      gameId.hidden = false;
      description.value = drafts.get(game.id) ?? game.description;
      form.hidden = false;
      for (const entry of list.querySelectorAll("[aria-current]"))
        entry.removeAttribute("aria-current");
      button.setAttribute("aria-current", "true");
      syncButtons();
      description.focus();
    });
    list.append(button);
  }
}

async function json(response) {
  const data = await response.json();
  if (!response.ok) throw new Error(data.error || "Request failed");
  return data;
}

search.addEventListener("input", renderGames);
description.addEventListener("input", () => {
  drafts.set(selected.id, description.value);
  syncButtons();
});
reset.addEventListener("click", () => {
  drafts.delete(selected.id);
  description.value = selected.description;
  syncButtons();
});

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (submit.disabled) return;
  const { id } = selected;
  const text = description.value;
  submitting = true;
  submitted.delete(id);
  syncButtons();
  status.textContent = `Opening a pull request for ${id}…`;
  try {
    const result = await json(
      await fetch("/api/submit", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id, description: text, sha }),
      })
    );
    const link = document.createElement("a");
    link.href = result.url;
    link.textContent = "View pull request on GitHub";
    link.target = "_blank";
    link.rel = "noopener";
    status.replaceChildren(`Pull request opened for ${id}. `, link);
    submitted.set(id, text);
  } catch (error) {
    status.textContent = error.message;
  } finally {
    submitting = false;
    syncButtons();
  }
});

try {
  const data = await json(await fetch("/api/catalog"));
  games = data.games.sort((left, right) => left.title.localeCompare(right.title, "he"));
  sha = data.sha;
  description.maxLength = data.maxLength;
  status.textContent = "";
  renderGames();
} catch (error) {
  status.textContent = `Could not load the catalog. Run gh auth login, then reload. ${error.message}`;
}
