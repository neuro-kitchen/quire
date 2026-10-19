# quire

A reference manager written in Rust: a local library of references and PDFs, BibTeX/BibLaTeX and CSL-JSON
import and export, and metadata lookup by DOI. A quire is a gathered set of pages; here, a gathered set of sources.

## Status
First milestone (2026-10-18): a CLI that manages projects, adds references by DOI, arXiv ID, PMID, PMCID or
URL, imports `.bib` files, and exports BibLaTeX, BibTeX or CSL-JSON. No PDFs, sync, TUI or GUI yet.

```bash
cargo build -p quire-cli                      # binary: target/debug/quire
quire init                                    # library + contact email (asked for by Crossref/NCBI)
quire project new Thesis
quire project new "Chapter 2" --parent thesis
quire add 10.1038/nature12373 arXiv:1706.03762 -p thesis
quire add pmid:31452104 https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0173392 -p thesis/chapter-2
quire list -p thesis -r
quire export -p thesis -r -o refs.bib         # --format bibtex | csl-json
```

## Layout (as in lumen and dsp-kitchen)
| Path | Purpose |
|---|---|
| `crates/quire-model` | Domain types, no I/O: IDs, identifiers, references, creators, dates, projects |
| `crates/quire-io` | CSL-JSON and BibTeX/BibLaTeX import/export (via Typst's `biblatex`) |
| `crates/quire-core` | Use cases (add, dedup, projects, export selection), cite keys, the `Library` port |
| `crates/quire-store` | SQLite storage: schema, migrations, `Library` implementation |
| `crates/quire-fetch` | Metadata lookup: doi.org, NCBI, web-page tags; rate-limited HTTP |
| `crates/quire-engine` | Facade for every front end: config, paths, async API |
| `crates/quire-cli` | The `quire` command line |
| `quire_py/` | Python SDK: PyO3 crate (`src/`, native module `quire_bindings`) + Python package `quire/`, tests |
| `playground/` | Experiments and small test programs |
| `docs/` | Documentation |
| `.knowledge/` | Notes, decisions, references |
| `assets/` | Icons, fonts, themes |

## Toolchain
Rust 1.99 stable (`rust-version = "1.99"`).

## Python (maturin + uv)
`pyproject.toml` builds `quire_py` with maturin (as in lumen): native module `quire_bindings.quire_bindings`,
Python package `quire` (Python 3.13, `.python-version`).

```bash
uv sync                       # builds the Rust extension via maturin and installs it (editable)
uv run python -c "import quire; print(quire.__version__)"
uv run maturin develop --uv   # rebuild after Rust changes (add --release for speed)
uv run pytest -q quire_py/tests
```

## Name
`quire` is taken on crates.io (a YAML parser) and reserved on PyPI; it is fine for local use and prefixed crate
names (`quire-core`, ...). Choose a distinct published name before any release.

## License
MIT OR Apache-2.0.
