package dev.neurokitchen.quire.domain.model

/** A reference a search found, best first; [snippet] shows where ([inText]: in the paper's text). */
data class SearchHit(val reference: Reference, val score: Double, val snippet: String, val inText: Boolean)

data class SavedSearch(val name: String, val query: String)

/** A search run against [source] (`arxiv`, `pubmed`, `crossref`) for new work; [checked] `YYYY-MM-DD`. */
data class Alert(val name: String, val source: String, val query: String, val checked: String?)

/** What checking an alert found: works not in the library yet, as previews ([Reference.id] empty). */
data class AlertResult(val alert: Alert, val found: List<Reference>, val error: String?)

/** Which neighbours of a reference in the citation graph. */
enum class GraphKind { References, Citing, Related }

/** A work next to a reference; [inLibraryId] its reference ID if it is in the library. */
data class GraphWork(
    val id: String,
    val doi: String?,
    val title: String?,
    val year: Int?,
    val firstAuthor: String?,
    val citedBy: Long,
    val inLibraryId: String?,
)
