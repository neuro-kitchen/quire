# Architecture

quire is a Cargo workspace. All behaviour lives in library crates; the command line, the
terminal interface and the local server are thin front ends over one facade, `quire-engine`.

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
| `quire_py` | Python bindings (PyO3, built with maturin) |

Dependencies point one way: `model ← core ← {store, io, fetch} ← engine ← {cli, tui, server}`.
The core defines what it needs from storage as a trait, so use cases are tested without a
database, and the store can change without touching them.

## Building and testing

```bash
cargo build                      # target/debug/quire
cargo test --workspace
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
