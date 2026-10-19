// The popup: saves the open tab (its HTML, or the PDF it shows) or an identifier to quire.
import { add, describe, projects, savePage, savePdf, saveSettings, settings, status } from "./api.js";

const $ = (id) => document.getElementById(id);

async function start() {
  $("open-options").addEventListener("click", () => chrome.runtime.openOptionsPage());
  const current = await settings();
  if (!current.token) {
    $("setup").hidden = false;
    return;
  }
  try {
    const info = await status();
    $("library").textContent = info.library;
    for (const project of await projects()) {
      const option = new Option(project.path, project.path);
      option.selected = project.path === current.project;
      $("project").add(option);
    }
    $("main").hidden = false;
  } catch (error) {
    show(error.message, true);
    return;
  }
  $("project").addEventListener("change", () => saveSettings({ project: $("project").value }));
  const tab = await activeTab();
  const pdf = await isPdf(tab);
  $("save").textContent = pdf ? "Save this PDF" : "Save this page";
  $("save").addEventListener("click", () =>
    run(async () => (pdf ? savePdf(tab.url, project()) : savePage(tab.url, await html(tab), project()))),
  );
  $("add-form").addEventListener("submit", (event) => {
    event.preventDefault();
    const input = $("input").value.trim();
    if (input) run(() => add(input, project()));
  });
}

const project = () => $("project").value || undefined;

async function activeTab() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  return tab;
}

/** Whether the tab shows a PDF (the browser's viewer may refuse scripts: then by its address). */
async function isPdf(tab) {
  try {
    const [{ result }] = await chrome.scripting.executeScript({ target: { tabId: tab.id }, func: () => document.contentType });
    return result === "application/pdf";
  } catch {
    return /\.pdf($|[?#])/i.test(tab.url);
  }
}

/** The page's HTML as the browser has it (after your login and any "are you human" check). */
async function html(tab) {
  const [{ result }] = await chrome.scripting.executeScript({
    target: { tabId: tab.id },
    func: () => document.documentElement.outerHTML,
  });
  return result;
}

async function run(action) {
  show("Saving…");
  $("issues").replaceChildren();
  try {
    const result = await action();
    show(describe(result));
    for (const issue of result.issues || []) {
      const item = document.createElement("li");
      item.textContent = issue.code + (issue.detail ? `: ${issue.detail}` : "");
      $("issues").append(item);
    }
    if (result.warning) show(`${describe(result)} (${result.warning})`, true);
  } catch (error) {
    show(error.message, true);
  }
}

function show(text, problem = false) {
  $("result").textContent = text;
  $("result").className = problem ? "problem" : "";
}

start();
