# Search

```bash
quire search transformer attention
quire search '"optical coherence"' author:smith
quire search tag:to-read note:dataset
```

The words must all match, as word beginnings (`trans` finds *transformer*). Quotes search a
phrase. A field name limits a word to that field: `title:`, `author:`, `abstract:`, `journal:`,
`tag:`, `note:`, `key:`, `doi:`. Results are ranked with title and authors counting most and the
papers' full text least; each shows where it matched.

The search covers metadata, tags, notes, highlights and every paper's extracted text. Texts are
indexed when they are extracted; `quire search --reindex` indexes texts extracted before search
existed.

In the TUI, `Ctrl-f` runs the same search.
