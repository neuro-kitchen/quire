# Terminal interface and reader

```bash
quire tui
```

Three panes: the library (all references, projects, tags, *Duplicates*, *Trash*), the
references of the selected view, and the details of the selected reference. `?` shows every key;
the [Keys](../reference/keys.md) page lists them too.

![The library: projects on the left, references with their year, authors and PDF pages, the selected reference's details and abstract](../images/tui/tui_main.png)

Common actions work on the selected reference: `a` adds by identifier, `p` gets the PDF, `o` opens
it, `t` toggles a tag, `1`–`9` toggle the colored tags, `e` edits in `$EDITOR`, `F` refetches,
`L` links to a project, `X` exports the current view, `Y` copies the cite key (through the
terminal, OSC 52: works over SSH; in tmux set `set -g set-clipboard on`). Downloads and lookups run in the
background; `J` lists them.

`/` filters the list as you type (fuzzy). `Ctrl-f` searches everything, papers' text included,
and sorts by relevance; the details pane shows where it matched.

## Reader

`Enter` on a reference opens its abstract first; `→` moves on to the paper. The text is reflowed
to one column, with tabs along the top:

![A paper open on its abstract, with its tabs: Paper (from Europe PMC), Figures, Tables, Equations, References](../images/tui/tui_paper-opened.png)

| Tab | Contents |
|---|---|
| Abstract | the abstract from the metadata |
| Paper | the body, with headings; `t` shows the contents |
| Front matter | authors, affiliations, journal and copyright lines taken out of the body |
| Figures, Tables | captions, numbered; `f` on a reference to a figure in the text jumps to it; on terminals that show images (kitty, WezTerm, Ghostty, foot and others with sixel, iTerm2), the figure at the top of the tab is shown beside it (arXiv papers) |
| Equations | numbered display equations |
| References | the paper's bibliography |
| Notes | your notes and highlights on this paper |

![The Tables tab: each table with its caption, in the sepia theme](../images/tui/tui-render.png)

Tabs only appear when the paper has something for them. Equations are shown with Unicode symbols
(`α`, `∑`, `x²`). `/` searches the text, `n`/`N` move between matches.

Layout: `+`/`-` change the column width, `J` toggles justified text, `L` the line spacing, `T` the
colors; defaults are the `reader.*` [settings](../reference/config.md). `R` extracts the text
again (see [PDFs](pdfs.md#text)).

![The same abstract in the sepia theme, justified, with double line spacing](../images/tui/tui-theme.png)

## Highlights

`v` starts selecting at the sentence on screen: `j`/`k` move to the next or previous sentence,
`l`/`h` make the selection longer or shorter. Then `m` highlights it, `c` highlights it with a
comment, `x` removes a highlight there.

A highlight is stored as the quoted text plus a little of what precedes it, not as a position,
so it survives the text being extracted again. Highlighting a passage again updates its comment.
From the command line:

```bash
quire highlight list smith2020      # Markdown quotes with their comments, ready to paste
quire highlight comment <id> -m "contradicts Table 2"
quire highlight rm <id>
quire highlight export -o ~/vault/papers   # one Markdown file per paper, for Obsidian or Logseq
```

`export` writes each paper with highlights or notes as `<cite key>.md`: its details as front
matter (title, authors, year, DOI, tags), then the highlights as quotes with your comments, then
the notes. Running it again rewrites only the files that changed; `-p thesis` limits it to a
project.

`a` in the reader writes a note on the paper.
