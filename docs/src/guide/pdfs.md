# PDFs

## Downloading

```bash
quire pdf get smith2020
quire pdf get -p thesis -r      # every reference of the project without a PDF
quire pdf get --all
```

Sources are tried in order: the PDF link the publisher's page declared when the reference was
added (`citation_pdf_url`), arXiv for preprints, Unpaywall's open-access copies, then the
publisher's landing page (which works for open-access journals, and on a campus network for
subscribed ones). PubMed Central copies need a browser (`--browser`, below).

Every file is checked before it is attached: that it really is a PDF (not a "verify you are
human" page saved as `.pdf`), whether it has text or is a scan, and whether the DOI it prints
matches the reference.

## Paywalls and bot checks: `--browser`

```bash
quire pdf get --blocked --browser   # every reference an earlier `pdf get` could not download
```

With `--browser`, quire still tries the download first. Only what fails is opened, one at a
time, in your own browser, where you log in or pass the check and download the PDF as usual.
quire watches your downloads folder (`access.downloads`, default `~/Downloads`), checks the new
file and attaches it. quire never sees your passwords or cookies.

If your institution has a login proxy, set it and pages open through it:

```bash
quire config set access.proxy "https://login.proxy.example.edu/login?url="
```

`--direct` opens publishers' pages without the proxy (useful when a bot check fails on the proxy
address); `--wait` sets how many minutes to wait for each download.

## Importing files you already have

```bash
quire pdf import ~/Downloads/*.pdf -p thesis   # finds the DOI or arXiv ID in each and adds the paper
quire pdf import paper.pdf --to smith2020      # attach to a known reference
```

## Other commands

```bash
quire pdf list --missing        # references without a PDF
quire pdf open smith2020
quire pdf path smith2020        # for scripts
quire pdf verify                # re-check every attached PDF against its reference
quire pdf rm smith2020          # detach (the file stays in the store)
```

## Text

quire extracts each paper's text for the reader and for search. It uses, in order, arXiv's HTML
version or the Europe PMC full text (both keep equations and figure captions intact), then the
PDF itself. PDFs whose text layer is hidden (some scans) are revealed; badly encoded ones are
retried with `pdftotext` when it is installed.

```bash
quire pdf text smith2020            # the text as the reader shows it (Markdown)
quire pdf text smith2020 --refresh  # extract again
quire pdf extract -p thesis         # extract again for many, e.g. after updating quire
```
