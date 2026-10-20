# Search, saved searches, reading list

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

## Filters

```bash
quire search status:unread pdf:yes
quire search transformer year:2019..2021 type:article-journal
quire search in:thesis/chapter-2 status:reading
```

Filters narrow a search, or make one on their own: `status:unread|reading|read`, `year:2019` or
`year:2019..2021` (`year:..2000`, `year:2020..`), `pdf:yes|no`, `type:` (a CSL type such as
`article-journal`, `book`, `chapter`), `in:` (a project and its subprojects).

## Saved searches

```bash
quire search status:unread pdf:yes --save "To read"
quire saved list
quire saved run "to read"
quire saved rename "to read" Queue
quire saved rm queue
```

A saved search runs again each time, so it works as a smart project: references that match
later show up in it. In the TUI, saved searches are in the left pane (`⌕`); `w` saves the open
search.

## Reading status

```bash
quire ref mark reading vaswani2017
quire ref mark read vaswani2017 kucsko2013
```

Opening a paper in the TUI reader marks it *reading*; `m` in the list cycles unread → reading →
read (shown `◐` and `✓`). quire keeps when you started and finished. Reading status and saved
searches are yours: they sync to your own devices, never to people you share a project with.
