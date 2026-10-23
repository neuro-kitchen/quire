# First steps

```bash
quire init                     # creates the library; asks for your contact email
```

The email is sent to Crossref, NCBI and Unpaywall with each request, as their terms ask; it lets
them contact you instead of blocking you. It is stored in `~/.config/quire/config.toml`.

## Add some papers

```bash
quire add 10.1038/nature12373 arXiv:1706.03762 pmid:31452104
quire list
```

`list` prints a table fitted to your terminal: cite key, first author, year, title, and marks for
the PDF and for metadata issues. `--json` prints JSON for scripts.

## Organise them

```bash
quire project new Thesis
quire project new "Chapter 2" --parent thesis
quire ref link vaswaniAttentionAllYou2017 -p thesis/chapter-2   # a subproject by its path
quire list -p thesis -r          # the project and its subprojects
```

## Get the PDFs and read

```bash
quire pdf get -p thesis -r       # open-access copies
quire tui                        # browse; Enter opens the reader
```

## Cite from a document

```bash
cd ~/papers/thesis
quire folder new -p thesis -r -o refs.bib
quire add 10.1093/nar/gkaa1100   # inside the folder: added to `thesis`, refs.bib rewritten
```

The next chapters cover each part in detail; [Commands](reference/cli.md) lists every option.
