# Desktop app

The desktop app shows the same library as `quire` and `quire tui`, in windows laid out like
Zotero 7: projects and tags on the left, the references in the middle, the selected one on the
right, and a tab for each paper you open. It reads and writes through the same engine and the
same files. Changes made in the app show everywhere in it at once; changes made outside it (`quire`
in a terminal, a sync) show after *View → Reload Library* (`Ctrl+R`).

The app is early: browsing, reading, highlighting, notes and settings work; adding and editing
references, PDFs, export and sync are still done with `quire` (see [What is not in the app
yet](#what-is-not-in-the-app-yet)).

![The library: projects and tags, the references, the selected reference's details](../images/app/desktop-library.png)

## Running it

The app is built from source for now (`app/` in the repository). You need Rust (as for `quire`)
and a JDK 21 or newer; Gradle and every other dependency are downloaded by the build.

```bash
cd app
./gradlew :desktopApp:run                                  # builds the engine, then opens the window
./gradlew :desktopApp:createDistributable -Pquire.release  # a self-contained app folder:
                                                           # desktopApp/build/compose/binaries/main/app/quire
```

The first build compiles the Rust engine and downloads PDFium (the PDF engine of Chrome, used for
the reader's pages), checking it against a pinned checksum. The app opens your default library
(`library.default`, see [Libraries and projects](libraries.md)) with your usual config file.

## The library

- **Left**: *My Library*, your projects (click the arrow to show subprojects), *Unfiled Items*
  (references in no project) and the *Trash*. Below them the tags of the references in view:
  click a tag to keep only references carrying it, click it again to drop it; the field under
  them filters the tag list.
- **Middle**: the toolbar's search field searches titles, creators, years, tags, notes and the
  papers' text (the same search as `quire search`). Click a column header to sort by it, again to
  reverse. A red PDF icon marks references with a PDF. Double-click a reference to read it.
- **Right**: the selected reference. Its sections (Info, Abstract, Attachments, Notes, Projects,
  Tags, Related) fold with a click on their title; the icons on the far right jump to them.
  *Open in Reader* or a click on the PDF opens the paper.

The window follows your system's light or dark theme.

## Reading

A paper opens in its own tab. With a PDF it opens on the pages; *Text* shows the paper's text in a
readable column, split like the terminal reader into the abstract, the paper, and tabs for its
figures, tables, equations and references. The text comes from arXiv or Europe PMC when they have
the paper, else from the PDF (see [Terminal interface and reader](tui.md)).

![A PDF with a highlight; annotations on the left, notes on the right](../images/app/desktop-reader-pdf.png)

- **Highlight**: in the *Text* view, select words in a paragraph and press *Highlight*. The
  highlight is marked in the text and on the PDF's pages, and listed on the left, where you can add
  a comment, remove it, or click it to bring it into view. Highlights are the same as the terminal
  reader's, and `quire highlight export` writes them to Markdown.
- **Zoom**: `−`, `+` and *Fit* in the PDF view.
- **Reading status**: opening a paper marks it *Reading*; *Unread*, *Reading* and *Read* on the
  right of the toolbar change it (as `quire ref mark`).
- **Notes**: the panel on the right lists the paper's notes; write one and press *Add note*.

![The text view, with a highlight](../images/app/desktop-reader-text.png)

## Settings

*Edit → Settings…* (`Ctrl+,`) opens the settings window: one section per group of settings
(General, Libraries, Sync, Export, Reader) and the file itself, *config.toml*.

![Settings: the Reader section](../images/app/desktop-settings.png)

- Every setting `quire config keys` lists has a row, with what it does and its key. Text and
  numbers apply with *Enter* or *Apply*; switches and choices apply at once. A value quire does not
  accept (an email without `@`, a width out of range) is refused with the reason under the row.
  *Reset* removes the setting, back to quire's default.
- Settings with a name in them (export profiles `export.<name>.…`, libraries, sync peers) show one
  row per name already in the file; add a new name in *config.toml*.
- *config.toml* shows the file as text. It is checked as you type (syntax errors, unknown keys) and
  saved whole or not at all; *Revert* goes back to the file as saved. Your comments and layout are
  kept by both the form and the editor.

After a change the library is opened again, so the new settings apply at once.

## What is not in the app yet

These work with `quire` (and `quire tui`) and appear in the app as soon as they are done there:
adding references (by identifier, from `.bib`, PDF or Zotero), editing fields, tags and projects,
getting PDFs, exports, duplicates, saved searches, alerts and sync. Phones (Android, iOS) and a web
version come later, on the same engine.
