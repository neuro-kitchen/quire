# Contributing to Quire

Thank you for your interest in contributing to **Quire**.

## License and Inbound Contribution Agreement

Quire's source code is licensed under the [GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later)](LICENSE), with brand and trademark terms defined in [TRADEMARK.md](TRADEMARK.md).

Because Quire is distributed both as open-source software and as official pre-built binaries through commercial application stores (including the Google Play Store and Apple App Store, whose platform terms impose distribution rules that require permission from copyright holders beyond the standard AGPL-3.0 terms), every contribution is accepted under the following inbound grant:

By submitting a pull request, patch, issue, or other contribution to this repository, you represent and agree that:

1. **Originality:** The contribution is your original work (or you have the legal right to submit it under these terms).
2. **License Grant:** You grant the project owner (`jesusdpa1` / NeuroKitchen) and all recipients of Quire a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable copyright and patent license to use, reproduce, modify, prepare derivative works of, publicly display, sublicense, and distribute your contribution:
   - under the **AGPL-3.0-or-later** as part of the open-source repository, **and**
   - as part of official binary distributions of Quire on any platform or application store (including Google Play, Apple App Store, and desktop distribution channels), under whatever terms those stores require.

## Code Quality Standards

Before opening a pull request, ensure all checks pass locally:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For GUI changes in `app/quire-gui`:

```bash
cd app/quire-gui && ./gradlew :shared:desktopTest
```
