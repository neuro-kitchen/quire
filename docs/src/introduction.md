# quire

quire is a reference manager for people who write in the terminal: Typst, LaTeX, Markdown. A
*quire* is a gathered set of pages; here, a gathered set of sources.

- **Add papers by identifier**: DOI, arXiv ID, PubMed ID, PMC ID or a URL. quire tries several
  sources in turn (doi.org, Crossref, arXiv, NCBI, the page's own citation tags) and records what
  failed, so gaps are visible.
- **Keep metadata right**: `quire ref check` lists problems; `ref refetch` updates what came
  from the web and never overwrites what you edited; duplicates (a preprint and its journal
  version) are found and merged.
- **PDFs**: open-access copies are downloaded and checked (is it a PDF? the right paper?). For
  paywalled ones, quire opens the page in your own browser and picks up the file you download.
- **Read**: `quire tui` browses the library; its reader shows a paper as reflowed text with
  tabs for figures, tables, equations and references, plus highlights and notes. The
  [desktop app](guide/desktop.md) does the same in windows laid out like Zotero, with the PDF's
  pages too.
- **Cite**: export BibLaTeX, BibTeX, CSL-JSON or Hayagriva (Typst). A document folder can keep its
  `.bib` updated after every change, so `typst watch` picks up new references as you add them.
- **Write**: check a manuscript's citations against the library (`quire folder check`).
- **Move from Zotero**: items, collections, tags, notes, PDFs and highlights (`quire import ~/Zotero`).
- **Discover**: what a paper cites and who cites it (OpenAlex), alerts on arXiv, PubMed and Crossref,
  retraction checks; saved searches and a reading list.
- **Search** metadata, notes, highlights and the papers' full text.
- **Sync** a library between your devices, directly and end-to-end encrypted, or through a
  [sync hub](guide/sync-hub.md) (your own, or a hosted one) that stores only data it cannot read;
  share projects with other people.

Everything is local: one SQLite database and a folder of PDFs per library.

![The library in the terminal](images/tui/tui_main.png)

![The desktop app](images/app/desktop-library.png)

quire is free software under the [GNU Affero General Public License v3.0 or later](https://www.gnu.org/licenses/agpl-3.0.html).
