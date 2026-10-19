# quire browser extension

Saves the paper you are looking at to your local quire library, through `quire serve`:

- **Save this page**: sends the page's HTML as your browser shows it (after your login and any
  "are you human" check), so quire reads its citation tags without fetching the page itself.
- **Save this PDF** (when the tab shows a PDF): downloads it with your browser's cookies and adds
  the paper it is.
- **Add**: a DOI, arXiv ID, PMID, PMCID or URL.
- Right-click: **Save linked PDF to quire**, **Save this page to quire**.

One Manifest V3 extension for Firefox (140+) and Chromium browsers (Chrome, Edge, Brave), plain
JavaScript with no build step.

## Install (development)

1. Run `quire serve` and keep it running; copy the token from `quire serve --token`.
2. Load the extension:
   - Firefox: `about:debugging#/runtime/this-firefox` → **Load Temporary Add-on** → pick
     `extension/manifest.json`.
   - Chromium: `chrome://extensions` → **Developer mode** → **Load unpacked** → pick `extension/`.
3. Open the extension's options, paste the token (port 23150 unless you changed it), **Save**.

## Safety

Nothing leaves your computer: the extension talks only to quire on 127.0.0.1 (declared to
Firefox as no data collection).

The server listens on 127.0.0.1 only and refuses requests without the token, so web pages cannot
use it. The token stays in the extension's local storage. The extension only reads the tab you
save, when you click; linked PDFs from other sites need the optional "all sites" permission,
asked for in the options.

## Files

`manifest.json`, `api.js` (every server call), `popup.*`, `options.*`, `background.js` (context
menu), `style.css`.
