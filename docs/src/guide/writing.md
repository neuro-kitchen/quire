# Writing with quire

## Checking a manuscript: `quire folder check`

In a document folder with `quire.toml` (see [Export and documents](export.md)):

```bash
quire folder check
```

quire reads the cite keys of every `.typ`, `.tex` and `.md` file in the folder (or the files you
name) and reports:

- **not in the library**: typos and papers never added, with the keys you probably meant;
- **not in the project**: in the library but missing from the folder's bibliography file;
  `--link` adds them to the project, and the file is rewritten;
- **never cited** (`--uncited` lists them): the project's references the text does not use.

It understands Typst (`@key`, `#cite(<key>)`), LaTeX (`\cite{a,b}` and the natbib and biblatex
variants: `\citep`, `\parencite`, `\textcites`, …) and Pandoc Markdown (`@key`, `[@a; -@b]`).
Comments and code are skipped, and so are references to the document's own labels
(`@fig-results` pointing to `<fig-results>`).

The exit status is 1 while a cited key is missing, so the check can run in CI or a pre-commit
hook.
