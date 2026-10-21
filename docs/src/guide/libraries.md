# Libraries and projects

## Libraries

A library is a folder: `quire.sqlite` (the references, projects, tags, notes and highlights) and
`files/` (PDFs, stored once each under their BLAKE3 hash). `quire init` creates the default one.

```bash
quire library new lab --path ~/shared/lab     # another library; registered by name
quire library list                            # * marks the default
quire library use lab                         # make it the default
quire --lib lab list                          # or just once (also QUIRE_LIB=lab)
quire --library ./some/folder list            # a folder that is not registered
quire ref copy smith2020 --to lab             # copy references between libraries
```

`library rm` only forgets the name; the folder and its files stay.

## Projects

Projects group references; a reference can be in any number of them, and projects nest.

```bash
quire project new Thesis
quire project new "Chapter 2" --parent thesis   # slug: thesis/chapter-2
quire project tree
quire ref link smith2020 jones2021 -p thesis/chapter-2
quire ref unlink smith2020 -p thesis/chapter-2  # stays in the library
quire list -p thesis -r                          # -r: with subprojects
```

Removing a project moves it to the trash; its references stay in the library. A project with
subprojects is removed only with `-r` (`quire project rm thesis -r`), which removes them too.

## Trash

`quire ref rm` moves references to the trash, `quire list --trash` shows it, and `quire ref
restore` brings them back.

## Backup and restore

```bash
quire library backup                          # quire-<library>-<date>.tar.gz here
quire library backup -o ~/backups/thesis.tar.gz
quire library restore ~/backups/thesis.tar.gz thesis-copy
```

A backup is one `.tar.gz` with a consistent copy of the database (taken while quire may be in
use), the PDFs and the papers' texts. It is not encrypted: keep it somewhere private. A restored
library is a new library (registered under the name you give) with a new device ID, so it can
sync with the library it came from without being mistaken for it.
