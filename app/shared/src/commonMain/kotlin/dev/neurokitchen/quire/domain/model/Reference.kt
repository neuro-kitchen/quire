package dev.neurokitchen.quire.domain.model

/** A reference as the app shows it; the full record lives in the Rust engine. */
data class Reference(
    val id: String,
    val citeKey: String,
    /** CSL type: `article-journal`, `book`, `paper-conference`, … */
    val kind: String,
    val title: String,
    val creators: List<Creator>,
    val year: Int?,
    /** As far as known: `1952`, `1952-08`, `1952-08-28`. */
    val date: String = year?.toString().orEmpty(),
    /** Journal, proceedings or book it appeared in. */
    val containerTitle: String = "",
    val publisher: String = "",
    val volume: String = "",
    val issue: String = "",
    val pages: String = "",
    val doi: String = "",
    val url: String = "",
    val abstract: String = "",
    val tags: List<String> = emptyList(),
    /** Paths of the projects it is in (`thesis/chapter-2`). */
    val projects: List<String> = emptyList(),
    val hasPdf: Boolean = false,
    val reading: ReadingStatus = ReadingStatus.Unread,
    /** `YYYY-MM-DD`. */
    val dateAdded: String = "",
)

data class Creator(
    /** Family name (with particle), or an organisation's whole name. */
    val sortName: String,
    /** "Family, Given". */
    val displayName: String = sortName,
    /** CSL role: `author`, `editor`, … */
    val role: String = "author",
)

enum class ReadingStatus { Unread, Reading, Read }
