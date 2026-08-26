// Manual pairing UI: a Chrome extension can't read config/secret.token off
// disk directly (../docs/protocol.md#trust-model--authentication), so the
// user copies it in here once and it's kept in chrome.storage.local from
// then on. background.js reconnects automatically as soon as it's saved.
const tokenInput = document.getElementById("token");
const status = document.getElementById("status");

async function load() {
  const { keylexToken } = await chrome.storage.local.get("keylexToken");
  if (keylexToken) tokenInput.value = keylexToken;
}

document.getElementById("save").addEventListener("click", async () => {
  const token = tokenInput.value.trim();
  await chrome.storage.local.set({ keylexToken: token });
  status.textContent = "Saved.";
  setTimeout(() => (status.textContent = ""), 2000);
});

load();
