# quire

A reference manager written in Rust: a local library of references and PDFs, BibTeX/BibLaTeX and CSL-JSON
import and export, and metadata lookup by DOI. A quire is a gathered set of pages; here, a gathered set of sources.

## Status
Empty Cargo workspace (2026-10-18). Crates are added under `crates/quire-<part>`.

## Layout (as in lumen and dsp-kitchen)
| Path | Purpose |
|---|---|
| `crates/` | Workspace crates `quire-*` (planned: core, io, fetch, cli, app) |
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
