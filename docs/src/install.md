# Install

quire is built from source for now (release binaries will follow). It needs Rust 1.99 or newer
([rustup](https://rustup.rs)).

```bash
git clone https://github.com/neuro-kitchen/quire
cd quire
cargo install --path crates/quire-cli     # installs `quire` into ~/.cargo/bin
```

SQLite is built in; there are no system libraries to install. Optional programs:

| Program | Used for |
|---|---|
| `pdftotext` (poppler) | a second try at the text of PDFs the built-in extractor reads badly |
| a web browser | PDFs behind a login or a bot check (`quire pdf get --browser`) |

## Shell completions

```bash
quire completions fish > ~/.config/fish/completions/quire.fish
quire completions bash > ~/.local/share/bash-completion/completions/quire
quire completions zsh  > ~/.zfunc/_quire       # with `fpath+=~/.zfunc` in .zshrc
```

## Secrets

The browser-extension token and the sync device key are kept in the system keyring (Secret
Service, Keychain, Credential Manager). Where there is none, set `QUIRE_NO_KEYRING=1` and they are
stored in files next to the config, readable only by you.
