# Browser extension

The extension saves the paper you are looking at into your library, through your own browser
session: if you can see the page or the PDF (logged in, past a bot check), quire gets it as you
see it.

- **Save this page**: sends the page as your browser shows it; quire reads its citation tags.
- **Save this PDF**: when the tab shows a PDF, downloads it with your browser and adds the paper.
- **Add** an identifier: DOI, arXiv ID, PMID, PMCID or URL.
- A project picker; right-click menus to save a linked PDF or the page.

It works in Firefox 140+ and Chromium browsers (Chrome, Edge, Brave).

## Setup

1. Start the local endpoint and keep it running:
   ```bash
   quire serve
   quire serve --token      # the token to give the extension
   ```
2. Load the extension from the `extension/` folder of the repository:
   - Firefox: `about:debugging#/runtime/this-firefox` → *Load Temporary Add-on* →
     `extension/manifest.json`.
   - Chrome, Edge, Brave: `chrome://extensions` → *Developer mode* → *Load unpacked* →
     `extension/`.
3. In the extension's options, paste the token and *Save*.

## Safety

The endpoint listens only on `127.0.0.1` and refuses requests without the token, so websites
cannot use it. The extension talks to nothing else and only reads a tab when you click save.
Saving linked PDFs from other sites needs the optional "all sites" permission, asked for in the
options. `quire serve --reset-token` makes a new token.
