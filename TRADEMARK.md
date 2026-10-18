# Trademark and Brand Policy

While the **Quire** source code is open source under the [GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later)](LICENSE), the **Quire** and **NeuroKitchen** names, logos, icons, and package identifiers are **trademarks** and are **not** licensed under the AGPL-3.0.

Pursuant to **Section 7(e) of the GNU Affero General Public License v3.0**, no rights are granted under trademark law for the use of the trade names, trademarks, service marks, logos, or application identifiers of Quire or NeuroKitchen.

---

## 1. Protected Assets ("The Marks")

The following assets are reserved trademarks and proprietary brand identifiers of the project authors (`jesusdpa1` / NeuroKitchen):

1. **Names & Wordmarks:** `"Quire"`, `"NeuroKitchen"`, `"Neuro-Kitchen"`.
2. **Application & Package Identifiers:**
   - `dev.neurokitchen.quire`
   - `dev.neurokitchen.*`
3. **Logos & Icons:**
   - All brand files under `assets/brand/` (including `quire-logo*`).
   - Launcher and application icons under `app/quire-gui/androidApp/src/main/res/mipmap*/`, `app/quire-gui/desktopApp/src/main/resources/icons/`, `app/quire-gui/shared/src/commonMain/composeResources/drawable/quire_logo*`, and `extension/icons/`.

---

## 2. What You May Do Without Permission

- **Build for personal or internal use:** You may clone this repository and build the CLI, TUI, hub, desktop app, or mobile app from source for your own personal devices or internal lab/organisation use without removing the logos or changing the package identifier.
- **Referential use:** You may use the name "Quire" truthfully in plain text to state that your software is based on, compatible with, or forked from Quire (for example, *"MyRefManager — a fork of Quire"*), provided you do not imply official endorsement or affiliation.
- **Unmodified source redistribution:** You may mirror or fork the source repository on code-hosting platforms (such as GitHub, GitLab, or Codeberg) in accordance with the AGPL-3.0-or-later.

---

## 3. What Requires Rebranding (App Stores & Public Binary Distribution)

Official pre-built binaries on commercial app stores (such as the **Google Play Store** and **Apple App Store**) are published by NeuroKitchen to fund ongoing development of the open-source project.

If you distribute compiled binaries, installers, containers, or packages (`.apk`, `.aab`, `.ipa`, `.dmg`, `.msi`, `.deb`, `.rpm`, Flatpak, Snap, F-Droid builds, etc.) to the public or upload them to **any third-party app store or repository**, you **MUST**:

1. **Change the name:** Do not use `"Quire"` or `"NeuroKitchen"` as the product or app title.
2. **Replace all logos and icons:** Remove all files in `assets/brand/` and replace all launcher/application icons with your own distinct artwork.
3. **Change the application ID:** Replace `dev.neurokitchen.quire` with your own domain namespace.
4. **State the origin clearly:** Comply with the AGPL-3.0-or-later by providing the complete corresponding source code and clearly stating that your distribution is an unofficial build or fork.
