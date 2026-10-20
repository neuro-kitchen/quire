# Adding and fixing references

## Adding

```bash
quire add 10.1038/nature12373          # DOI
quire add arXiv:1706.03762             # arXiv (old-style IDs too: hep-th/9901001)
quire add pmid:31452104 PMC7096803     # PubMed, PubMed Central
quire add https://www.nature.com/articles/s41586-020-2649-2
quire import refs.bib -p thesis        # a BibTeX / BibLaTeX file
```

Each identifier goes through a chain of sources until one answers: DOI content negotiation →
Crossref → the source's own API (arXiv, NCBI) → the page's JSON-LD → its `citation_*` tags.
Every failed attempt is recorded with the reference, so `ref check` can show what went wrong.

Cite keys are made from the first author, the title and the year (`vaswaniAttentionAllYou2017`)
and stay fixed once made.

## Checking

```bash
quire ref check                 # all references with issues
quire ref check -p thesis -r
```

Issues include a missing year or authors, an unknown type, a title in capitals, a failed lookup,
a page blocked by the publisher, possible duplicates, and (with `--retractions`, see
[Finding and watching papers](discover.md)) retractions. `quire list` marks references with
errors `!` and warnings `~`.

## Fixing

```bash
quire ref edit smith2020        # opens the reference as TOML in $EDITOR
quire ref refetch smith2020     # look it up again; shows what changed
quire ref refetch -p thesis --dry-run
quire ref abstracts             # fill in missing abstracts (Europe PMC, else the PDF)
```

Every field remembers where it came from. `refetch` only replaces fields that came from a lookup:
what you edited is kept.

## Duplicates

A preprint and its journal version, or the same paper added twice, are found by a shared
identifier, or by the same title (ignoring case and punctuation) with a compatible first author
and a year at most two apart.

```bash
quire ref duplicates
quire ref merge smith2020 smith2019a   # keep the first; the others go to the trash
quire ref distinct smith2020 smith2019a   # not duplicates: stop suggesting them
```

Merging moves projects, tags, notes, highlights, PDFs and identifiers to the reference kept. In
the TUI, the *Duplicates* view lists them with `≈`; `M` merges or marks them distinct.

## Related papers

`quire ref relate a b` links two references both ways (a comment and its reply, a paper and its
dataset); `ref unrelate` removes the link.
