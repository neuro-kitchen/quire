# Architecture

quire is a Cargo workspace. All behaviour lives in library crates; the command line, the
terminal interface, the local server and the apps are thin front ends over one facade,
`quire-engine`.

| Crate | Purpose |
|---|---|
| `quire-model` | domain types, no I/O: IDs, identifiers, references, creators, dates, projects |
| `quire-io` | formats: CSL-JSON, BibTeX/BibLaTeX, Hayagriva, the edit format; PDF checks and text |
| `quire-core` | use cases (adding, duplicates, search queries, sync rows) and the `Library` port |
| `quire-store` | SQLite: schema, migrations, the `Library` implementation, full-text index, change log |
| `quire-fetch` | metadata, PDF and full-text lookup: doi.org, Crossref, arXiv, NCBI, Europe PMC, Unpaywall |
| `quire-engine` | the facade: config, libraries, file store, exports, jobs; async API |
| `quire-sync` | sync sessions and the iroh (QUIC) transport |
| `quire-server` | `quire serve`, the endpoint for the browser extension |
| `quire-tui` | `quire tui` (ratatui) |
| `quire-cli` | the `quire` binary |
| `quire-ffi` | the engine for the Kotlin apps (UniFFI): flat records, `suspend` functions, PDF pages (PDFium) |
| `uniffi-bindgen` | generates `quire-ffi`'s Kotlin API (run by the app's Gradle build) |
| `quire_py` | Python bindings (PyO3, built with maturin) |

Dependencies point one way: `model ← core ← {store, io, fetch} ← engine ← {cli, tui, server, ffi}`.
The core defines what it needs from storage as a trait, so use cases are tested without a
database, and the store can change without touching them.

How a paper's text is read (blocks, tabs for figures, tables, equations, references) lives in
`quire_io::reading`, shared by the terminal reader and the apps.

## The apps (`app/`)

A Kotlin Compose Multiplatform project: one UI for the desktop now, Android and iOS later. The
Gradle build compiles `quire-ffi`, generates its Kotlin API with `uniffi-bindgen`, and puts the
native library in the app (JNA loads it). PDFium, for the reader's pages, is downloaded from
bblanchon/pdfium-binaries and checked against pinned SHA-256 sums.

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
cd app
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
