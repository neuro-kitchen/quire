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
  tabs for figures, tables, equations and references, plus highlights and notes.
- **Cite**: export BibLaTeX, BibTeX, CSL-JSON or Hayagriva (Typst). A document folder can keep its
  `.bib` updated after every change, so `typst watch` picks up new references as you add them.
- **Search** metadata, notes, highlights and the papers' full text.
- **Sync** a library between your devices, directly and end-to-end encrypted, with no server
  holding your data.

Everything is local: one SQLite database and a folder of PDFs per library.
