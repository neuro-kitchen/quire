# Moving from Zotero

```bash
quire import ~/Zotero                      # your Zotero data folder
quire import ~/Zotero -p from-zotero       # collections go under one project
quire import ~/Zotero --base ~/Papers      # where linked files live, if you use linked files
```

quire reads a copy of `zotero.sqlite`, so Zotero may stay open. From "My Library" (not group
libraries; nothing in the trash) it takes:

- **items** as references: type, fields, authors and editors, date, DOI, and the PMID, PMCID or
  arXiv ID written in *Extra*. The *Citation Key* (Zotero 7, or Better BibTeX's line in *Extra*)
  is kept when it is free.
- **collections** as projects, subcollections as subprojects; an item in several collections
  is linked to each project (one reference).
- **tags** (automatic ones stay automatic), **notes** (as text), and **related** items.
- **PDFs** stored in Zotero (`storage/`), checked like any PDF quire downloads; linked files when
  their folder is known (absolute paths, or `--base` for paths relative to Zotero's "Linked
  Attachment Base Directory").
- **highlights and underlines** from Zotero's PDF reader become highlights (with their comments);
  note annotations become notes with their page. Image and ink annotations are not imported.

A work already in quire (same DOI or other identifier) is linked, not duplicated. Importing again
adds only what is new: items are recognised by their Zotero key (also those without a DOI),
notes by their text, highlights by their passage.
