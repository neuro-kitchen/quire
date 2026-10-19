// The local quire server (`quire serve`): settings and every call the extension makes.
// The server listens on 127.0.0.1 only and needs the token `quire serve --token` prints.

export const DEFAULT_PORT = 23150;

/** { port, token, project } from the extension's storage. */
export async function settings() {
  const stored = await chrome.storage.local.get(["port", "token", "project"]);
  return { port: stored.port || DEFAULT_PORT, token: stored.token || "", project: stored.project || "" };
}

export async function saveSettings(values) {
  await chrome.storage.local.set(values);
}

/** A JSON (or error) answer from the server; throws with its message on failure. */
async function call(path, init = {}) {
  const { port, token } = await settings();
  if (!token) throw new Error("No token yet: run `quire serve --token` and paste it in the options.");
  let response;
  try {
    response = await fetch(`http://127.0.0.1:${port}${path}`, {
      ...init,
      headers: { ...(init.headers || {}), Authorization: `Bearer ${token}` },
    });
  } catch {
    throw new Error(`quire is not running on port ${port}: start it with \`quire serve\`.`);
  }
  const body = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(body.error || `quire answered ${response.status}`);
  return body;
}

export const status = () => call("/v1/status");
export const projects = () => call("/v1/projects");

/** Adds a DOI, arXiv ID, PMID, PMCID or URL. */
export const add = (input, project) =>
  call("/v1/add", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ input, project }) });

/** Adds the paper a page shows, from its HTML (what the browser already loaded). */
export const savePage = (url, html, project) =>
  call("/v1/page", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ url, html, project }) });

/** Downloads `url` with the browser's cookies (your logins) and adds the paper the PDF is. */
export async function savePdf(url, project) {
  const download = await fetch(url, { credentials: "include" });
  if (!download.ok) throw new Error(`the PDF could not be downloaded (${download.status})`);
  const bytes = await download.arrayBuffer();
  const query = new URLSearchParams({ url });
  if (project) query.set("project", project);
  return call(`/v1/pdf?${query}`, { method: "POST", headers: { "Content-Type": "application/pdf" }, body: bytes });
}

/** One line about what the server did. */
export function describe(result) {
  if (result.reference) {
    const verb = result.status === "added" ? "Added" : "Already in the library:";
    return `${verb} ${result.reference.cite_key}`;
  }
  if (result.cite_key) return `${result.new ? "Saved" : "Already had"} the PDF of ${result.cite_key}`;
  return "Done";
}
