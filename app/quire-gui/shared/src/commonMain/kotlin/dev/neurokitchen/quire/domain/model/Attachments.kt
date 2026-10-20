package dev.neurokitchen.quire.domain.model

/**
 * Whether a PDF is the reference's paper, as the engine judged it from its first pages.
 * [Unknown]: nothing to compare (a scan); [Different]: it prints another paper's identifier.
 */
enum class PdfVerdict { Identifier, Title, Unknown, Different, Unlikely }

/** A PDF attached to [reference]; [isNew] `false` if it already had this exact file. */
data class AttachedPdf(val reference: Reference, val isNew: Boolean, val path: String, val verdict: PdfVerdict)

/** What getting a PDF did: downloaded from [source], or the [files] it already had. */
data class PdfFetch(val downloaded: Boolean, val source: String?, val url: String?, val files: List<String>, val verdict: PdfVerdict?)

/** A reference's PDF: available (with its page count), or why the last download found none. */
sealed interface PdfState {
    data class Available(val pages: Int?) : PdfState
    /** [reason]: `blocked` (a bot check or login: try the browser), `not-found`, `no-identifier`, `error`. */
    data class Missing(val reason: String) : PdfState
}

/** A stored PDF checked again against its reference. */
data class PdfCheck(val referenceId: String, val path: String, val verdict: PdfVerdict)

/** A PDF file added as a reference (by the identifier it prints) and attached to it. */
data class ImportedPdf(val added: AddOutcome, val attached: AttachedPdf)
