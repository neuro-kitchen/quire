# Privacy Policy (`quire` & Hosted Sync Hub)

**Last updated:** October 23, 2026

**Quire** is built by **NeuroKitchen** around a strict local-first, privacy-by-design architecture. Your academic library, PDFs, highlights, and notes belong to you and live on your devices.

This Privacy Policy explains what data is processed when you use the Quire applications (CLI, TUI, Desktop, Android, iOS, and Browser Extension) and when you optionally use our hosted synchronization service (`quire-hub` and `iroh-relay`).

---

## 1. Local-First Applications (CLI, TUI, Desktop, Android, iOS)

When you use Quire locally without connecting to a sync hub:

1. **Zero Telemetry, Analytics, or Ad SDKs:** The Quire applications contain **no** analytics trackers, crash-reporting telemetry, behavioral profiling, or advertising SDKs. We do not collect your reading history, search queries, library contents, or device identifiers.
2. **Local Storage Only:** Your SQLite library database (`library.sqlite`), configuration (`config.toml`), and PDF files are stored locally in your device's application directory or a folder you choose.
3. **Direct Third-Party Academic Lookups:** When you explicitly ask Quire to look up metadata, citations, or open-access PDFs (for example via `quire add`, `quire ref check`, `quire pdf get`, `quire alert`, or the GUI's "Add by identifier" / citation graph tools), your device connects **directly** to public scholarly APIs, which may include:
   - **DOI Foundation (`doi.org`) & Crossref (`api.crossref.org`)**
   - **OpenAlex (`api.openalex.org`)**
   - **arXiv (`export.arxiv.org`)**
   - **NCBI / PubMed / Europe PMC (`eutils.ncbi.nlm.nih.gov`)**
   - **Unpaywall (`api.unpaywall.org`)**
   If you configure a contact email (`user.email` / `quire init`), Quire includes that email in the `User-Agent` or `mailto` parameter of requests to Crossref, OpenAlex, NCBI, and Unpaywall solely to comply with those public APIs' "polite pool" rate-limit policies. Those requests go directly from your device to the respective academic service and never pass through NeuroKitchen servers.

---

## 2. Optional Peer-to-Peer Sync (`quire sync`)

When you sync directly between your own devices (`quire sync pair` / `quire sync now`), your devices establish an end-to-end encrypted QUIC connection via `iroh`. If a direct peer-to-peer path cannot be established across NAT/firewalls, encrypted QUIC packets are relayed through an `iroh-relay` server. The relay sees only ephemeral connection IP addresses and public `NodeId` keys and cannot decrypt the traffic.

---

## 3. Optional Hosted Sync Hub (`quire-hub`)

If you choose to use our hosted `quire-hub` cloud service (or self-host your own), your devices encrypt all library data **before** it leaves your device.

### 3.1 What Is Encrypted Client-Side (Unreadable by Us)
Before uploading to `quire-hub`, your device seals all library content using `XChaCha20-Poly1305` authenticated encryption and keyed `BLAKE3` content hashes (`crates/quire-sync/src/crypto.rs`):
- Reference metadata (titles, authors, DOIs, abstracts, BibTeX/CSL fields)
- Projects, tags, saved searches, and reading lists
- Personal notes, Markdown annotations, and PDF highlights
- Attached PDF and document files
- Collaborator identity cards and project invitations (`key_envelope` and `device_box` payloads are sealed to recipient public keys)

Because the encryption keys live exclusively on your devices and in your personal Recovery Kit (`quire user recovery-kit`), **NeuroKitchen cannot read, index, search, or decrypt your library contents or PDFs.**

### 3.2 Operational Metadata Stored on `quire-hub`
To operate the sync service and enforce storage quotas, the `quire-hub` SQLite database (`crates/quire-hub/src/store.rs`) stores only the minimum required structural metadata:

| Data Category | Exact Fields Stored | Purpose |
|---|---|---|
| **Account record** | Internal numeric `account` ID, storage `quota_bytes`, creation timestamp (`created_at`), one-time link code hash | Enforcing storage quota and linking your devices |
| **Registered devices** | Device Ed25519 public key (`device` / `NodeId`), sealed device name (`name_sealed`), registration and last-seen timestamps (`added_at`, `last_seen_at`) | Authenticating QUIC sync sessions and showing you your linked devices |
| **Encrypted sync records** | Random library/author UUIDs, sequence numbers, keyed-BLAKE3 file hashes, ciphertext byte sizes (`size`), upload timestamps | Ordering sync batches, deduplicating sealed files within your account, and enforcing quota |
| **Server logs** | Aggregate counts (active sessions, batch counts, byte sizes) and connection errors | Service health monitoring (`RUST_LOG=quire_hub=info,warn`); logs never contain library content |
| **Billing records** *(paid hosted plans only)* | Subscription status and payment transaction IDs managed by our Merchant of Record / payment processor (or Google Play / Apple App Store for mobile purchases) | Account provisioning, tax compliance, and fraud prevention |

---

## 4. Data Storage, Backups, and International Transfers (GDPR / CCPA)

1. **Hosting & Backup Locations:** Our hosted `quire-hub` server and encrypted S3-compatible object storage backups (Litestream WAL replication and `rclone sync` of sealed file blobs) may be hosted in the European Union or the United States depending on the region selected at deployment.
2. **Cross-Border Transfers & Safeguards:** Where operational metadata or client-sealed ciphertext of European Economic Area (EEA), UK, or Swiss users is transferred outside the EEA/UK/Switzerland, we rely on the European Commission's **Standard Contractual Clauses (SCCs)** with our infrastructure sub-processors, supplemented by mandatory client-side encryption (`XChaCha20-Poly1305`) before data leaves your device.
3. **Legal Basis under GDPR:**
   - **Performance of a Contract (`Art. 6(1)(b) GDPR`):** Processing operational metadata and storing your sealed blobs to provide the sync service you requested.
   - **Legitimate Interests (`Art. 6(1)(f) GDPR`):** Securing the `quire-hub` and `iroh-relay` against abuse and enforcing storage quotas.
   - **Legal Obligation (`Art. 6(1)(c) GDPR`):** Retaining tax and billing records via our payment processor where required by law.
4. **No Sale or Sharing of Personal Information (CCPA/CPRA):** We do **not** sell, rent, or share your personal information or operational metadata with third parties for cross-context behavioral advertising or marketing.

---

## 5. Automated Data Purging & Right to Erasure ("Right to be Forgotten")

Under the GDPR (`Art. 17`), CCPA, and similar privacy laws, you have the right to access, export, and permanently delete your data:

1. **Data Portability:** Because Quire is local-first, your complete unencrypted library is already stored on your device and can be exported at any time to standard open formats (`BibLaTeX`, `BibTeX`, `CSL-JSON`, `Hayagriva YAML`, or a portable `.quire-backup` archive).
2. **Hard Deletion (`quire-hub account delete`):** When you delete your hosted account (or request erasure):
   - Every row associated with your account (`account`, `device`, `device_activity`, `code`, `library`, `author`, `batch`, `snapshot`, `file`, `device_box`, `device_author`, `expected_device`, and `key_envelope`) is immediately deleted from the `quire-hub` SQLite database, followed by `VACUUM` and `PRAGMA wal_checkpoint(TRUNCATE)` so freed database pages are zeroed/truncated and replicated to backup storage;
   - Your entire encrypted file directory (`/data/files/<account>`) is immediately deleted from the server disk; and
   - Our automated backup synchronization (`rclone sync`) purges deleted account blobs from the remote object-storage bucket so no orphaned encrypted files remain.

---

## 6. Contact Us

For privacy inquiries, GDPR/CCPA data deletion requests, or questions about this Privacy Policy, please contact us at:

- **Email:** `privacy@neurokitchen.dev`
- **Repository Issues:** `https://github.com/neuro-kitchen/quire/issues`
