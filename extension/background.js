// Context menu: save a linked PDF, or the page, without opening the popup.
import { describe, savePage, savePdf, settings } from "./api.js";

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({ id: "quire-link", title: "Save linked PDF to quire", contexts: ["link"] });
  chrome.contextMenus.create({ id: "quire-page", title: "Save this page to quire", contexts: ["page"] });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  const { project } = await settings();
  try {
    let result;
    if (info.menuItemId === "quire-link") {
      // A link may point to another site: that needs the optional "all sites" permission.
      const granted = await chrome.permissions.contains({ origins: ["<all_urls>"] });
      if (!granted) throw new Error("Allow quire to download from all sites in its options first.");
      result = await savePdf(info.linkUrl, project);
    } else {
      const [{ result: html }] = await chrome.scripting.executeScript({
        target: { tabId: tab.id },
        func: () => document.documentElement.outerHTML,
      });
      result = await savePage(tab.url, html, project);
    }
    notify(tab.id, describe(result));
  } catch (error) {
    notify(tab.id, `quire: ${error.message}`);
  }
});

/** Shows `text` on the extension's icon for the tab (no notification permission needed). */
function notify(tabId, text) {
  chrome.action.setTitle({ tabId, title: text });
  chrome.action.setBadgeText({ tabId, text: text.startsWith("quire:") ? "!" : "✓" });
}
