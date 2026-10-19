// Options: the token and port of `quire serve`, checked against the running server.
import { saveSettings, settings, status } from "./api.js";

const $ = (id) => document.getElementById(id);

async function start() {
  const current = await settings();
  $("token").value = current.token;
  $("port").value = current.port;
  $("form").addEventListener("submit", async (event) => {
    event.preventDefault();
    await saveSettings({ token: $("token").value.trim(), port: Number($("port").value) });
    try {
      const info = await status();
      show(`Connected to library “${info.library}”.`);
    } catch (error) {
      show(error.message, true);
    }
  });
  $("all-sites").addEventListener("click", async () => {
    const granted = await chrome.permissions.request({ origins: ["<all_urls>"] });
    show(granted ? "Linked PDFs can be saved." : "Not allowed.", !granted);
  });
}

function show(text, problem = false) {
  $("result").textContent = text;
  $("result").className = problem ? "problem" : "";
}

start();
