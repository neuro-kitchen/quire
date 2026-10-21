# Tags and notes

## Tags

```bash
quire tag add smith2020 to-read methods   # one reference, any number of tags
quire tag color to-read blue --key 1   # in the TUI, key 1 toggles it
quire list -t to-read -t methods       # references with both tags
quire tag rename to-read queue
quire tag delete queue                 # removes it from every reference
```

Tag names ignore case (`Methods` and `methods` are one tag). Renaming a tag to the name of another
merges the two (`merged to-read → queue`), as in Zotero. Up to nine tags have a color, each on a
key 1–9; giving a tag a key another tag holds takes the key (and color) from that one.

## Notes

Notes are Markdown, attached to a reference or standalone.

```bash
quire note add smith2020 -m "Uses the 2018 dataset; compare with Table 2."
quire note add smith2020               # without -m: your editor opens
quire note list smith2020
quire note edit <note-id>
```

In the reader, `a` writes a note on the paper you are reading; its *Notes* tab shows the notes and
highlights together. Highlights are covered in [the reader](tui.md#highlights).
