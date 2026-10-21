# Install

quire is built from source for now (release binaries will follow). It needs Rust 1.99 or newer
([rustup](https://rustup.rs)).

```bash
git clone https://github.com/neuro-kitchen/quire
cd quire
cargo install --path app/quire-cli        # installs `quire` into ~/.cargo/bin
```

SQLite is built in; there are no system libraries to install. Optional programs:

| Program | Used for |
|---|---|
| `pdftotext` (poppler) | a second try at the text of PDFs the built-in extractor reads badly |
| `ocrmypdf` | OCR of scanned PDFs: a text layer added to the PDF (`quire pdf ocr`) |
| `tesseract` and `pdftoppm` (poppler) | OCR without OCRmyPDF: the scan's text for the reader and search |
| a web browser | PDFs behind a login or a bot check (`quire pdf get --browser`) |

The sync hub (`quire-hub`) runs as a container; see [Sync hub](guide/sync-hub.md).

## The desktop app

The desktop app (`app/`) also builds from source: it needs, besides Rust, a JDK 21 or newer.

```bash
cd app/quire-gui
./gradlew :desktopApp:run                                   # build and open it
./gradlew :desktopApp:createDistributable -Pquire.release   # a self-contained app folder, with its own Java
```

See [Desktop app](guide/desktop.md).

## Shell completions

```bash
quire completions fish > ~/.config/fish/completions/quire.fish
quire completions bash > ~/.local/share/bash-completion/completions/quire
quire completions zsh  > ~/.zfunc/_quire       # with `fpath+=~/.zfunc` in .zshrc
```

## Secrets

The browser-extension token, the device's sync key and your user key (the same on all your
devices; `quire user recovery-kit` prints it to keep offline) are kept in the system keyring
(Secret Service, Keychain, Credential Manager). Where there is none (or with `QUIRE_NO_KEYRING=1`)
they are stored in files next to the config, readable only by you.
