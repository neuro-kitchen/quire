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

/** A `quire://hub?…` link (pasted or scanned): the hub's ID, our own relay, a one-time code. */
data class HubLink(val id: String, val relay: String?, val code: String?)

/** A device asking to be linked, with the right code: its name and the words both screens show. */
data class LinkRequest(val name: String, val words: List<String>)

/** This device offering to link another: [uri] shows as a QR code (two minutes, once). */
interface LinkOffer {
    val uri: String
    /** Waits for a device with the right code (others are refused); fails when the code expires. */
    suspend fun nextRequest(): LinkRequest
    /** Gives the waiting device this person's user key, hub and relay. */
    suspend fun allow()
    suspend fun refuse()
    suspend fun cancel()
}

/** This device joining another's code: compare [words], then [finish] waits for the other's decision. */
interface LinkJoin {
    val words: List<String>
    /** The user ID now in use, once allowed (reopen the library then). */
    suspend fun finish(): String
    suspend fun cancel()
}

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
