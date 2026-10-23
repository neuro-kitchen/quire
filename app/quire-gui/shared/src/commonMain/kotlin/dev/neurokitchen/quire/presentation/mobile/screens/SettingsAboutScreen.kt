package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsGroup
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsItem
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_info
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.ic_warning
import org.jetbrains.compose.resources.painterResource

/**
 * Settings › About & Legal screen:
 * - Application version & AGPL-3.0-or-later license header
 * - Mandatory Independent Backup & Zero-Knowledge Encryption Notice
 * - Expandable Legal summaries (Terms of Service, Privacy Policy, Trademark Policy)
 * - Searchable Open Source Licenses inventory for bundled Rust and Kotlin/Android libraries
 */
@Composable
internal fun AboutAndLegalPage(version: String) {
    var selectedDoc by rememberSaveable { mutableStateOf<LegalDocKind?>(null) }
    var searchQuery by rememberSaveable { mutableStateOf("") }

    selectedDoc?.let { doc ->
        LegalDocumentDialog(
            doc = doc,
            onDismiss = { selectedDoc = null },
        )
    }

    val filteredNotices = remember(searchQuery) {
        val q = searchQuery.trim().lowercase()
        if (q.isEmpty()) {
            OPEN_SOURCE_NOTICES
        } else {
            OPEN_SOURCE_NOTICES.filter {
                it.name.lowercase().contains(q) ||
                    it.license.lowercase().contains(q) ||
                    it.category.lowercase().contains(q)
            }
        }
    }

    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
    ) {
        SettingsBrandFooter(version = version)

        // 1. Backup & Zero-Knowledge Responsibility Notice
        SettingsGroup(
            label = "Data Safety & Encryption",
            rows = listOf { shape ->
                BackupNoticeCard(shape = shape)
            },
        )

        // 2. Legal Documents (Terms of Service, Privacy Policy, Trademark Policy, AGPLv3 Source)
        SettingsGroup(
            label = "Legal & Compliance",
            rows = LegalDocKind.entries.map { kind ->
                { shape: Shape ->
                    SettingsItem(
                        title = kind.title,
                        supporting = kind.subtitle,
                        icon = kind.icon,
                        shape = shape,
                        opensPage = true,
                        onClick = { selectedDoc = kind },
                    )
                }
            },
        )

        // 3. Open Source Licenses Attribution (MIT, Apache-2.0, BSD, MPL-2.0, Zlib, etc.)
        SettingsGroup(
            label = "Open Source Licences (${filteredNotices.size})",
            rows = buildList {
                add { shape ->
                    Column(
                        Modifier
                            .fillMaxWidth()
                            .clip(shape)
                            .background(QuireTheme.colors.cardBackground)
                            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceSm),
                        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
                    ) {
                        Text(
                            text = "Quire statically links and bundles open-source Rust and Kotlin Multiplatform libraries under permissive licences.",
                            style = MaterialTheme.typography.bodySmall,
                            color = QuireTheme.colors.fillSecondary,
                        )
                        OutlinedTextField(
                            value = searchQuery,
                            onValueChange = { searchQuery = it },
                            placeholder = {
                                Text(
                                    "Filter crates or libraries…",
                                    color = QuireTheme.colors.fillTertiary,
                                )
                            },
                            leadingIcon = {
                                androidx.compose.material3.Icon(
                                    painter = painterResource(Res.drawable.ic_search),
                                    contentDescription = null,
                                    tint = QuireTheme.colors.fillSecondary,
                                )
                            },
                            singleLine = true,
                            shape = RoundedCornerShape(Dimens.RadiusBase),
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }
                }
                filteredNotices.forEach { notice ->
                    add { shape ->
                        SettingsItem(
                            title = notice.name,
                            supporting = "${notice.category} · ${notice.license}\n${notice.copyright}",
                            icon = Res.drawable.ic_library_books,
                            shape = shape,
                        )
                    }
                }
            },
        )
    }
}

@Composable
private fun BackupNoticeCard(shape: Shape) {
    val colors = QuireTheme.colors
    Column(
        Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(colors.cardBackground)
            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceMd),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Text(
            text = "Maintain Independent Local Backups",
            style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.SemiBold),
            color = colors.fillPrimary,
        )
        Text(
            text = "Quire is a local-first reference manager. When using Sync or a Sync Hub, your references, notes, highlights, and PDFs are encrypted on your device (XChaCha20-Poly1305 & keyed BLAKE3) before upload.",
            style = MaterialTheme.typography.bodySmall.copy(lineHeight = 18.sp),
            color = colors.fillSecondary,
        )
        Text(
            text = "Because NeuroKitchen never holds your encryption keys, we cannot reset your keys or recover your library if you lose all linked devices and your Recovery Kit. Always keep independent local backups (`quire library backup`) and save your Recovery Kit (`Settings › Sync`).",
            style = MaterialTheme.typography.bodySmall.copy(lineHeight = 18.sp),
            color = colors.fillSecondary,
        )
    }
}

private enum class LegalDocKind(
    val title: String,
    val subtitle: String,
    val icon: org.jetbrains.compose.resources.DrawableResource,
    val body: String,
) {
    Terms(
        title = "Terms of Service & Warranty Disclaimer",
        subtitle = "AS-IS service, liability cap, local backups, Acceptable Use & DMCA policy",
        icon = Res.drawable.ic_description,
        body = """
            1. Open Source vs. Official App & Hosted Hub
            Quire's source code is licensed under GNU AGPL-3.0-or-later (see LICENSE in the repository). Official mobile app binaries and hosted sync hub accounts are provided by NeuroKitchen under these Terms (full text in TERMS.md).

            2. Mandatory Independent Backups & Recovery Kit
            You are solely responsible for maintaining independent local backups of your library and PDFs (`quire library backup`) and safely storing your Recovery Kit (`quire user recovery-kit`). Hosted sync uses client-side encryption (XChaCha20-Poly1305); if you lose your devices and Recovery Kit, hosted ciphertext is permanently unrecoverable.

            3. "AS-IS" Service & No Uptime SLA
            The software and hosted sync services are provided strictly "AS IS" and "AS AVAILABLE" without warranties of any kind, including any warranty of uninterrupted sync or protection against data loss or sync conflicts.

            4. Limitation of Liability
            To the maximum extent permitted by law, NeuroKitchen and its contributors are not liable for any indirect, consequential, or research-related damages (including lost data, annotations, or missed deadlines). Total cumulative liability is capped at the greater of the amount you paid NeuroKitchen in the preceding 12 months or $10.00 USD ($0.00 for free/self-hosted use).

            5. Acceptable Use Policy (AUP)
            You may not store classified data, ITAR/EAR export-controlled technical data, or HIPAA Protected Health Information on the hosted hub, nor use the hub as a public file mirror or abuse relay quotas.

            6. Legal Process & DMCA Safe Harbor (17 U.S.C. § 512(c))
            While library contents are sealed on your device, operational account metadata and sealed ciphertext blobs are subject to valid legal process. NeuroKitchen responds to statutory DMCA notices (`dmca@neurokitchen.dev`) and terminates repeat infringers.
        """.trimIndent(),
    ),
    Privacy(
        title = "Privacy Policy & GDPR / CCPA Rights",
        subtitle = "Zero telemetry, direct academic API lookups, client-side encryption & data erasure",
        icon = Res.drawable.ic_info,
        body = """
            1. Local-First & Zero Telemetry
            Quire contains no analytics, crash trackers, or advertising SDKs. Your SQLite library and PDFs live on your device.

            2. Direct Academic API Lookups
            When you look up DOIs, arXiv IDs, PMIDs, citation graphs, or open-access PDFs, your device connects directly to Crossref, OpenAlex, arXiv, NCBI/PubMed, and Unpaywall. If you configure a contact email (`Settings › Account`), it is sent directly to those APIs for their polite rate-limit pools and never passes through NeuroKitchen servers.

            3. Hosted Sync Hub Data Inventory
            Before upload to `quire-hub`, references, notes, highlights, and PDFs are encrypted on your device (`XChaCha20-Poly1305` and keyed `BLAKE3`). The hub stores only operational metadata: account ID, quota, device Ed25519 public keys, timestamps, ciphertext sizes/hashes, and sealed blobs.

            4. International Transfers & Right to Erasure (GDPR Art. 17 / CCPA)
            Cross-border transfers rely on Standard Contractual Clauses (SCCs) supplemented by client-side encryption. We never sell or share personal data. Deleting an account (`quire-hub account delete` or contacting `privacy@neurokitchen.dev`) immediately deletes all SQLite rows (`secure_delete = ON` + WAL truncate), deletes `/data/files/<account>`, and purges remote backup buckets via `rclone sync` so no orphan blobs remain.
        """.trimIndent(),
    ),
    Trademarks(
        title = "Source Code & Trademark Policy",
        subtitle = "GNU AGPL-3.0-or-later source licence and AGPLv3 §7(e) trademark notice",
        icon = Res.drawable.ic_warning,
        body = """
            1. Source Code (`AGPL-3.0-or-later`)
            The Quire source code is free software licensed under the GNU Affero General Public License v3.0 or later. You may inspect, modify, build from source, and self-host `quire-hub` under the terms of the AGPLv3.

            2. Reserved Trademarks (AGPL-3.0 Section 7(e))
            Pursuant to Section 7(e) of the GNU AGPLv3, the "Quire" and "NeuroKitchen" word marks, logos (`assets/brand/`), app icons, and application package identifiers (`dev.neurokitchen.quire`) are reserved trademarks of NeuroKitchen (`TRADEMARK.md`).

            3. Redistribution Requirement
            If you build and publicly distribute modified or unmodified binaries (including on Google Play, Apple App Store, F-Droid, or other repositories), you must remove the reserved Quire/NeuroKitchen branding and use your own distinct application name, icon, and package identifier.
        """.trimIndent(),
    ),
    PermissiveText(
        title = "Standard Open-Source Licence Texts",
        subtitle = "MIT, Apache-2.0, BSD-2/3-Clause, ISC, MPL-2.0, and Zlib conditions",
        icon = Res.drawable.ic_library_books,
        body = """
            MIT License
            Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, subject to the above copyright notice and this permission notice being included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND.

            Apache License, Version 2.0
            Licensed under the Apache License, Version 2.0 (the "License"); you may not use these components except in compliance with the License. You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0. Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

            BSD 2-Clause & 3-Clause Licenses
            Redistribution and use in source and binary forms, with or without modification, are permitted provided that the copyright notice, this list of conditions, and the following disclaimer are retained. THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES ARE DISCLAIMED.
        """.trimIndent(),
    ),
}

@Composable
private fun LegalDocumentDialog(
    doc: LegalDocKind,
    onDismiss: () -> Unit,
) {
    val colors = QuireTheme.colors
    DialogShell(
        title = doc.title,
        subtitle = doc.subtitle,
        icon = doc.icon,
        onDismiss = onDismiss,
        modifier = Modifier.padding(horizontal = Dimens.SpaceMd),
        actions = {
            DialogCancel(onClick = onDismiss)
        },
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(
                text = doc.body,
                style = MaterialTheme.typography.bodySmall.copy(
                    fontSize = 12.sp,
                    lineHeight = 18.sp,
                ),
                color = colors.fillSecondary,
            )
            Spacer(Modifier.height(4.dp))
            Text(
                text = "Full texts: LICENSE, TERMS.md, PRIVACY.md, TRADEMARK.md in github.com/neuro-kitchen/quire",
                style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
                color = colors.fillTertiary,
            )
        }
    }
}

private data class OpenSourceNotice(
    val name: String,
    val category: String,
    val license: String,
    val copyright: String,
)

private val OPEN_SOURCE_NOTICES = listOf(
    // First-party engine
    OpenSourceNotice("quire (quire-core, quire-engine, quire-sync, quire-document, quire-ffi)", "Rust Engine", "AGPL-3.0-or-later", "Copyright © 2026 NeuroKitchen"),
    // Kotlin & Android UI Libraries
    OpenSourceNotice("Kotlin Standard Library, Coroutines, Serialization & Datetime", "Kotlin / Multiplatform", "Apache-2.0", "Copyright © 2010–2026 JetBrains s.r.o. and Kotlin Programming Language contributors"),
    OpenSourceNotice("Compose Multiplatform & Jetpack Compose (Material 3, Foundation, UI, Navigation 3)", "UI Toolkit", "Apache-2.0", "Copyright © JetBrains s.r.o. and The Android Open Source Project"),
    OpenSourceNotice("AndroidX Activity, Lifecycle & Core", "Android Jetpack", "Apache-2.0", "Copyright © The Android Open Source Project"),
    OpenSourceNotice("Koin (koin-core, koin-android, koin-compose-viewmodel)", "Dependency Injection", "Apache-2.0", "Copyright © Kotzilla & InsertKoinIO contributors"),
    OpenSourceNotice("Haze (haze, haze-blur, haze-blur-materials)", "UI Blur Effects", "Apache-2.0", "Copyright © Chris Banes"),
    OpenSourceNotice("QRose", "QR Code Rendering", "MIT", "Copyright © Alexander Zhirkevich"),
    OpenSourceNotice("Java Native Access (JNA)", "Desktop Native FFI", "Apache-2.0 OR LGPL-2.1-or-later", "Copyright © Timothy Wall and JNA contributors"),
    // Rust Core, Sync, Crypto, Storage & PDF Crates
    OpenSourceNotice("UniFFI (uniffi, jni)", "Rust–Kotlin Bindings", "MPL-2.0 / MIT OR Apache-2.0", "Copyright © Mozilla Corporation and UniFFI contributors"),
    OpenSourceNotice("Tokio, Axum, Reqwest, Rustls & rustls-platform-verifier", "Async Runtime & Networking", "MIT / Apache-2.0 / ISC", "Copyright © Tokio Contributors, Sean McArthur, and Rustls Contributors"),
    OpenSourceNotice("iroh", "Peer-to-Peer QUIC Transport & Relay", "MIT OR Apache-2.0", "Copyright © n0, Inc."),
    OpenSourceNotice("RustCrypto (chacha20poly1305) & curve25519-dalek", "Client-Side Authenticated Encryption", "Apache-2.0 OR MIT / BSD-3-Clause", "Copyright © RustCrypto Developers & dalek-cryptography contributors"),
    OpenSourceNotice("BLAKE3", "Cryptographic Content Hashing", "CC0-1.0 OR Apache-2.0", "Copyright © Jack O'Connor and Samuel Neves"),
    OpenSourceNotice("SQLite, rusqlite, rusqlite_migration & tokio-rusqlite", "Embedded Database", "MIT / Public Domain", "Copyright © SQLite Consortium & The rusqlite Developers"),
    OpenSourceNotice("hayro (hayro, hayro-interpret, hayro-syntax, hayro-cmap), lopdf & pdf-inspector", "Pure-Rust PDF Interpreter & Text Layer", "MIT OR Apache-2.0", "Copyright © Laurenz Vargas, J-F-Liu, and contributors"),
    OpenSourceNotice("biblatex & hayagriva", "Bibliography Parsing & Formatting", "MIT OR Apache-2.0", "Copyright © Typst GmbH"),
    OpenSourceNotice("Serde, serde_json, toml, toml_edit & prost", "Serialization & Protocol Buffers", "MIT OR Apache-2.0", "Copyright © Erick Tryzelaar, David Tolnay, and Tokio Contributors"),
    OpenSourceNotice("scraper, roxmltree, url, deunicode & unicode-normalization", "HTML/XML & Unicode Processing", "MIT OR Apache-2.0 / MPL-2.0", "Copyright © Servo Project Developers, Yevhenii Reizner, and Unicode, Inc."),
    OpenSourceNotice("flate2, zip, tar, image & qrcode", "Archives & Image Decoding", "MIT OR Apache-2.0 / Zlib", "Copyright © Rust Project Developers and contributors"),
    OpenSourceNotice("ratatui, crossterm, nucleo-matcher, clap & tabled", "CLI & Terminal UI", "MIT OR Apache-2.0", "Copyright © Ratatui Developers, Timon Post, Helix Editor, and Kevin K."),
)
