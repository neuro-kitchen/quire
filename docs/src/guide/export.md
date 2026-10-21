# Export and documents

## One-off exports

```bash
quire export -p thesis -r -o refs.bib                  # BibLaTeX (default)
quire export --format bibtex -o refs.bib
quire export --format hayagriva -o refs.yml            # Typst
quire export --format csl-json -o refs.json            # Pandoc, Zotero
```

Without `-o` the result goes to standard output.

A `.bib` file imported with `quire import` and exported again keeps its fields: those quire has no
place for (`keywords`, `file`, …) are stored with the reference and written back, and LaTeX math in
titles (`$T^2$`) stays math. An entry that names one paper twice (`pmid` and a PubMed `eprint`) is
read with that identifier once. A preprint is written as `@online` with its arXiv `eprint` (what
Biber's styles expect; classic BibTeX gets `@misc`), or as `@article` in `arXiv` with
`preprints = article` (what Typst expects).

## Files that stay up to date

```bash
quire export -p thesis -r -o ~/papers/thesis/refs.bib --keep-updated
quire export auto           # files kept updated
quire export stop ~/papers/thesis/refs.bib
```

After any change that touches the project (a reference added, a title fixed), the file is
rewritten. With `typst watch` or `latexmk -pvc`, the document follows along.

## Document folders: `quire.toml`

```bash
cd ~/papers/thesis
quire folder new -p thesis -r -o refs.bib
```

This writes `quire.toml` in the folder. Inside it (and its subfolders), `quire add` puts new
references into `thesis`, `quire export` and `quire list` work on that project without flags, and
`refs.bib` is kept updated. `quire folder show` prints the settings that apply; `--all` works on
the whole library instead.

A Typst document then only needs:

```typ
#bibliography("refs.bib")
```

## Profiles

A profile fixes how a format is written; set it in the config and use it with `--profile`:

```bash
quire config set export.typst.format biblatex
quire config set export.typst.preprints article      # Typst styles expect @article
quire config set export.typst.omit abstract,file,note
quire config set export.journal.format bibtex
quire config set export.journal.ascii true           # é → {\'e}
quire config set export.journal.months macro       # month = jul, not 7
quire export --profile typst -o refs.bib
```

See [Settings](../reference/config.md) for every key. To check that a document's citations are
all in its bibliography, see [`quire folder check`](writing.md#checking-a-manuscript-quire-folder-check).
