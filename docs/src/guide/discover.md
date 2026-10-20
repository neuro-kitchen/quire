# Finding and watching papers

## The citation graph

```bash
quire ref cites kucsko2013            # the works it cites
quire ref cited-by kucsko2013         # works citing it, most cited first
quire ref similar kucsko2013          # related works
quire ref cites kucsko2013 --add -p thesis
```

The graph comes from [OpenAlex](https://openalex.org). Works already in the library are marked
`●`; `--add` adds the others by DOI. No account is needed; looking a work up is free and each
list costs a fraction of a cent of OpenAlex's free daily budget.

## Alerts

```bash
quire alert add nv arxiv nitrogen-vacancy thermometry
quire alert add nv-pubmed pubmed 'nanodiamond[ti] AND thermometry'
quire alert add lukin crossref author:Lukin quantum sensing
quire alert check                     # every alert: what appeared since the last check
quire alert check nv --add -p thesis  # and add it
quire alert list
```

An alert searches arXiv, PubMed or Crossref. Plain words must all appear; each source's own
syntax works too (arXiv `au:` `ti:` `cat:`, PubMed `[au]` `[ti]`, Crossref `author:`). A check
lists the papers published since the last check (the first looks back 30 days, `--days` changes
it) that the alert has not shown before and that are not in the library. Alerts are kept on the
device where they were made.

## Retractions

```bash
quire ref check --retractions
```

asks Crossref, which includes the Retraction Watch database, about every reference with a DOI.
Retracted, withdrawn or removed papers get a `retracted` error, papers with an expression of
concern a warning; both show in `ref check`, as `!` / `~` in `quire list`, and in the TUI.
