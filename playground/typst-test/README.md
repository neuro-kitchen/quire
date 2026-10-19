# Typst test for quire

Typst documents citing references that quire looked up and exported, one per output:

| Document | Bibliography file | Written by | Style |
|---|---|---|---|
| `main.typ` | `refs.bib` (BibLaTeX) | `quire.toml` (profile `paper`, kept updated) | IEEE |
| `main-hayagriva.typ` | `refs.yml` (Hayagriva) | `quire export --format hayagriva -o refs.yml --keep-updated` | APA |
| `main-bibtex.typ` | `refs-bibtex.bib` (BibTeX, ASCII, month macros) | `quire export --profile classic -o refs-bibtex.bib --keep-updated` | Chicago author-date |

`body.typ` holds the shared text. The `.bib`/`.yml` files are checked in, so `typst compile main.typ`
works without quire.

To rebuild from scratch (Typst 0.15.1, quire from this repository):

```bash
# config.toml with the two profiles used here
quire config set export.paper.omit abstract
quire config set export.paper.preprints article      # Typst reads @online as a web page
quire config set export.classic.format bibtex
quire config set export.classic.omit abstract
quire config set export.classic.ascii true
quire config set export.classic.months macro

quire project new "Typst test" && quire project new Methods --parent typst-test
# here: quire.toml already exists (project typst-test, output refs.bib, profile paper)
quire add 10.1038/nature12373 arXiv:1706.03762 hep-th/9711200 pmid:31452104 PMC6710268 \
  https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0173392 \
  https://github.blog/news-insights/company-news/github-availability-report-september-2025/
quire add 10.1093/nar/gkaa1100 -p typst-test/methods
quire export --format hayagriva -o refs.yml --keep-updated
quire export --profile classic -o refs-bibtex.bib --keep-updated
typst compile main.typ && typst compile main-hayagriva.typ && typst compile main-bibtex.typ
```

Checked 2026-10-19: all three compile without warnings. Findings are in
`.knowledge/bibliography-formats.md` (Typst section).
