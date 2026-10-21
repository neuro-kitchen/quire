package dev.neurokitchen.quire.domain.model

/** What a sync through the hub did (`.knowledge/sync-hub.md`); [used] and [quota] in bytes (0: no limit). */
data class HubReport(
    val registered: Boolean,
    val joined: Boolean,
    val pushed: Int,
    val applied: Int,
    val filesUploaded: Int,
    val filesDownloaded: Int,
    /** PDFs not uploaded: the hub's storage is full. */
    val overQuota: Int,
    val used: Long,
    val quota: Long,
)

/** One of this person's devices the hub knows; times in milliseconds since 1970. */
data class HubDevice(val device: String, val firstSeen: Long, val lastSeen: Long, val revoked: Boolean)

/** Where an opened PDF came from. */
sealed interface PdfSource {
    data object Here : PdfSource
    data object Hub : PdfSource
    data class Peer(val name: String) : PdfSource
    data class Found(val source: String) : PdfSource
}

/** A PDF ready to read (fetched first if it was not on this device). */
data class OpenedPdf(val path: String, val hash: String, val source: PdfSource)

/** What removing PDFs past the cache limit did; bytes. */
data class Eviction(val removed: Int, val freed: Long, val kept: Long)
