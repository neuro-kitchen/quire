# Architecture

quire is a Cargo workspace. All behaviour lives in library crates; the command line, the
terminal interface, the local server and the apps are thin front ends over one facade,
`quire-engine`.

| Crate | Purpose |
|---|---|
| `quire-model` | domain types, no I/O: IDs, identifiers, references, creators, dates, projects |
| `quire-io` | bibliographic formats: CSL-JSON, BibTeX/BibLaTeX, Hayagriva, Zotero, the edit format |
| `quire-core` | use cases (adding, duplicates, search queries, sync rows) and the `Library` port |
| `quire-store` | SQLite: schema, migrations, the `Library` implementation, full-text index, change log |
| `quire-fetch` | the network: metadata, PDF and full-text lookup (doi.org, Crossref, arXiv, NCBI, Europe PMC, Unpaywall) |
| `quire-engine` | the facade: config, libraries, file store, exports, jobs; async API |
| `quire-sync` | sync sessions and the iroh (QUIC) transport |
| `quire-connector` | `quire serve`, the endpoint for the browser extension |
| `quire-document` | a paper's content in any format: PDF checks, rendering and text layer (hayro); PDF, arXiv HTML and JATS → the reader's Markdown; the reading model |
| `quire-ffi` | the engine for the Kotlin apps (UniFFI): flat records, `suspend` functions, PDF pages |
| `quire-testkit` | test fixtures (a tiny PDF, the services' recorded responses); dev-dependency only |
| `uniffi-bindgen` | generates `quire-ffi`'s Kotlin API (run by the app's Gradle build) |
| `app/quire-cli` | the `quire` binary |
| `app/quire-tui` | `quire tui` (ratatui) |
| `app/quire-gui` | desktop app (Compose Multiplatform); Android and iOS later |

Dependencies point one way: `model ← core ← {store, io, document, fetch, sync} ← engine ← {cli, tui, server, ffi}`.
The core defines what it needs from storage as a trait, so use cases are tested without a
database, and the store can change without touching them.

How a paper's text is read (blocks, tabs for figures, tables, equations, references) lives in
`quire_document::reading`, shared by the terminal reader and the apps.

## The apps (`app/`)

A Kotlin Compose Multiplatform project: one UI for the desktop now, Android and iOS later. The
Gradle build compiles `quire-ffi`, generates its Kotlin API with `uniffi-bindgen`, and puts the
native library in the app (JNA loads it). PDF pages and text layer are powered by pure-Rust hayro
(`crates/quire-document`), requiring no external binary downloads.

```
app/
  shared/      every screen and ViewModel (commonMain); the engine repositories (desktopMain)
  desktopApp/  the window, menus, packaging
  androidApp/  Android shell (not built yet)
```

Screens follow MVVM with one-way data flow: a screen draws its `State` and sends `Event`s, its
`ViewModel` changes the state. Each screen has its own folder (`Screen`, `State`, `Event`,
`ViewModel`, previews, and a `components/` folder for what only it uses); components shared by
several screens have a folder each. Rules stay in Rust: ViewModels call repositories, which call
the engine.

```bash
cd app/quire-gui
./gradlew :shared:desktopTest    # unit and engine tests; screenshots in shared/build/screenshots
./gradlew :desktopApp:run
```

## Building and testing

```bash
cargo build                      # target/debug/quire
cargo test --workspace           # QUIRE_PDFIUM_DIR=<folder of libpdfium> also tests PDF pages
cargo clippy --workspace --all-targets
cargo fmt --all
```

The same checks run on every push and pull request (`.github/workflows/ci.yml`).

## This book

The book is in `docs/` (mdBook). The [command](reference/cli.md) and
[settings](reference/config.md) references are generated from the binary; after changing a
command's help, regenerate them:

```bash
cargo build -p quire-cli
docs/gen-reference.sh target/debug/quire
mdbook serve docs                # preview at http://localhost:3000
```

CI fails when the generated pages are out of date, and publishes the book to GitHub Pages from
`main` (`.github/workflows/docs.yml`).
