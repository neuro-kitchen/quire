# Terms of Service (`quire` & Hosted Sync Hub)

**Last updated:** October 23, 2026

These Terms of Service ("Terms") govern your use of **Quire** software applications (including the command-line interface, terminal interface, browser extension, desktop application, and mobile applications for Android and iOS) and any hosted cloud synchronization or relay services ("Hosted Services", including the hosted `quire-hub` and `iroh-relay` endpoints) operated by **NeuroKitchen** ("we", "us", or "our").

By installing, running, purchasing, or connecting to Quire or the Hosted Services, you agree to these Terms.

---

## 1. Open Source Software vs. Hosted Services & Official Binaries

1. **Open Source License (`AGPL-3.0-or-later`):** The source code of Quire in the public repository is licensed under the [GNU Affero General Public License v3.0 or later](LICENSE). Nothing in these Terms restricts your rights under `AGPL-3.0-or-later` to inspect, modify, compile from source, or self-host your own instance of `quire-hub`.
2. **Trademarks & Branding:** Pursuant to Section 7(e) of the GNU AGPLv3, the **"Quire"** and **"NeuroKitchen"** names, logos, icons, and application package identifiers (`dev.neurokitchen.quire`) are reserved trademarks and are subject to our [Trademark & Redistribution Policy](TRADEMARK.md).
3. **Paid Mobile Apps & Hosted Sync:** Official pre-compiled mobile binaries distributed through app stores (e.g., Google Play Store, Apple App Store) and accounts on our Hosted Services are subject to these Terms, including the backup obligations, Acceptable Use Policy, and limitations of liability below.

---

## 2. Mandatory Independent Backups & Recovery Kit Responsibility

Academic libraries, manuscripts, notes, and annotations often represent years of irreplaceable research. **Quire is a local-first system, and you are solely responsible for backing up your data:**

1. **Maintain Independent Local Backups:** You must maintain independent, regular backups of your local library database and PDF collection (for example, using `quire library backup`, automated filesystem snapshots, or off-site backups). Do **not** rely on `quire-hub` or peer-to-peer synchronization as your sole backup.
2. **Client-Side Encryption & Unrecoverable Keys:** When you use `quire-hub` (hosted or self-hosted), your references, notes, highlights, and PDFs are sealed on your device before transmission (`XChaCha20-Poly1305` and keyed `BLAKE3` hashes) using keys held exclusively on your devices. **We do not hold your encryption keys and cannot decrypt your library or reset your encryption keys.**
3. **Recovery Kit (`quire user recovery-kit`):** You are solely responsible for exporting and safely storing your recovery kit. If you lose all linked devices and your recovery kit, any encrypted data stored on the Hosted Services is permanently and mathematically unrecoverable.

---

## 3. "AS-IS" Service & No Uptime Warranty (No SLA)

1. **Provided "AS-IS" and "AS AVAILABLE":** Unless you or your institution have executed a separate, written Enterprise Service Level Agreement (SLA) with NeuroKitchen, the Quire software and all Hosted Services are provided strictly **"AS IS"** and **"AS AVAILABLE"**, without warranties of any kind, whether express, implied, or statutory, including without limitation implied warranties of merchantability, fitness for a particular purpose, non-infringement, or error-free operation.
2. **No Guarantee Against Sync Conflicts or Data Loss:** We do not warrant that synchronization across devices or through `quire-hub` will be uninterrupted, timely, conflict-free, or free from data corruption or loss.

---

## 4. Limitation of Liability

TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE LAW:

1. **Exclusion of Consequential & Research Damages:** IN NO EVENT SHALL NEUROKITCHEN, ITS AUTHORS, CONTRIBUTORS, OR LICENSORS BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, OR EXEMPLARY DAMAGES, INCLUDING WITHOUT LIMITATION LOSS OF DATA, LOSS OF RESEARCH, LOSS OF MANUSCRIPTS OR ANNOTATIONS, MISSED PUBLICATION OR GRANT DEADLINES, WORK STOPPAGE, OR COMPUTER FAILURE OR MALFUNCTION, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.
2. **Liability Cap:** OUR TOTAL CUMULATIVE LIABILITY ARISING OUT OF OR RELATED TO THESE TERMS, THE SOFTWARE, OR THE HOSTED SERVICES—WHETHER IN CONTRACT, TORT (INCLUDING NEGLIGENCE), STRICT LIABILITY, OR OTHERWISE—SHALL NOT EXCEED THE GREATER OF:
   - **(a)** THE TOTAL AMOUNT YOU ACTUALLY PAID TO NEUROKITCHEN FOR THE OFFICIAL MOBILE APPLICATION OR HOSTED SERVICES IN THE TWELVE (12) MONTHS IMMEDIATELY PRECEDING THE EVENT GIVING RISE TO LIABILITY, OR
   - **(b)** TEN U.S. DOLLARS ($10.00 USD) (OR ZERO DOLLARS ($0.00) IF YOU USE ONLY FREE OR SELF-HOSTED DISTRIBUTIONS).

Some jurisdictions do not allow the exclusion of certain warranties or the limitation of liability for incidental or consequential damages; in such jurisdictions, our liability is limited to the maximum extent permitted by law.

---

## 5. Acceptable Use Policy (AUP)

When using our Hosted Services (`quire-hub` and `iroh-relay`), you agree **not** to:

1. **Prohibited Regulated Data:** Store, transmit, or process:
   - Classified government or defense information;
   - Technical data subject to International Traffic in Arms Regulations (**ITAR**) or Export Administration Regulations (**EAR**) restrictions;
   - Unanonymized Protected Health Information (**PHI**) regulated under HIPAA or equivalent clinical patient-identifiable records without a separate written Business Associate Agreement.
2. **Private Personal Research & Authorized Collaboration Only:** Use the Hosted Services as a public file-distribution mirror, public link-sharing CDN, or unauthorized mass distribution service for copyrighted publisher PDFs. Project sharing (`quire share`) may only be used among bona fide research collaborators within a private academic or laboratory group.
3. **No Infrastructure or Relay Abuse:** Attempt to bypass storage quotas (`QUIRE_HUB_QUOTA_MB`), flood the `iroh-relay` or `quire-hub` with excessive connections, reverse-engineer or probe other users' accounts, or use the Hosted Services for any unlawful purpose, malware distribution, or unauthorized network relaying.

We reserve the right to suspend or terminate accounts that violate this Acceptable Use Policy or exceed their contracted storage or bandwidth limits.

---

## 6. Encryption Scope & Compliance with Legal Process

1. **Technical Scope of Client-Side Encryption:** On our Hosted Services, library batches, snapshots, and attachment files are encrypted on your device prior to upload (`XChaCha20-Poly1305`, keyed `BLAKE3` content identifiers). Consequently, we cannot inspect, index, or read the plaintext titles, DOIs, notes, or PDF contents stored in your account.
2. **Unencrypted Operational Metadata:** To route connections and enforce quotas, our Hosted Services necessarily process operational metadata, including your account identifier, storage quota, registered device Ed25519 public keys (`NodeId`), connection timestamps, encrypted batch/file counts, byte sizes, and billing records (handled by our payment processor).
3. **Compliance with Valid Legal Process:** We make no representation that use of the Hosted Services makes you "anonymous" or immune from legal process. We reserve the right to comply with valid subpoenas, preservation requests, court orders, or search warrants by producing the operational account metadata and/or the client-sealed ciphertext blobs in our possession, and by suspending or terminating accounts where required by law.

---

## 7. Copyright & DMCA Safe Harbor Policy (`17 U.S.C. § 512(c)`)

1. **Storage at Direction of Users:** Our Hosted Services operate solely as an automated, encrypted storage and synchronization conduit at the direction of individual users. Because all stored files are encrypted client-side before upload, we have no ability to view, index, or verify the contents of stored blobs.
2. **DMCA Takedown Notices:** We respect intellectual property rights and comply with the Digital Millennium Copyright Act (`17 U.S.C. § 512`). If you are a copyright owner (or authorized agent) and believe that specific material stored on our Hosted Services infringes your copyright, you may submit a written notification pursuant to `17 U.S.C. § 512(c)(3)` to our Designated Copyright Agent containing:
   - A physical or electronic signature of a person authorized to act on behalf of the owner of an exclusive right that is allegedly infringed;
   - Identification of the copyrighted work claimed to have been infringed;
   - Identification of the material that is claimed to be infringing and information reasonably sufficient to permit us to locate the account or stored material;
   - Your contact information (address, telephone number, and email address);
   - A statement that you have a good-faith belief that use of the material in the manner complained of is not authorized by the copyright owner, its agent, or the law; and
   - A statement that the information in the notification is accurate, and under penalty of perjury, that you are authorized to act on behalf of the owner of an exclusive right that is allegedly infringed.
3. **Repeat Infringer Policy:** In accordance with the DMCA and other applicable laws, we have adopted a policy of terminating, in appropriate circumstances and at our sole discretion, the accounts of users who are deemed to be repeat infringers.
4. **Designated Agent Contact:** Notices under this section should be sent to the Designated Agent address published on our official website and registered with the U.S. Copyright Office (`dmca@neurokitchen.dev`).

---

## 8. Account Cancellation & Data Deletion

1. **Voluntary Deletion:** You may delete your Hosted Services account at any time. Upon account deletion (`quire-hub account delete`), all associated account rows, encrypted batches, snapshots, device registrations, and encrypted attachment blobs are permanently purged from active storage and synchronized out of our object-storage backups.
2. **Subscription Expiry:** If your paid Hosted Services subscription expires or fails renewal, we will retain your encrypted data for a grace period of at least thirty (30) days so you can renew or sync to a local device, after which your hosted account and encrypted blobs may be permanently deleted. Your local library on your devices remains intact at all times.

---

## 9. Changes to These Terms

We may update these Terms from time to time. Material changes will be noted by updating the "Last updated" date at the top of this document and in the repository release notes. Continued use of the Hosted Services after an update constitutes acceptance of the revised Terms.
