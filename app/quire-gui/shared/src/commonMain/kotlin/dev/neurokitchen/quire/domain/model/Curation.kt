package dev.neurokitchen.quire.domain.model

/** A reference and those that look like the same work. */
data class DuplicateGroup(val reference: Reference, val duplicates: List<Duplicate>)

/** [sharedIdentifier] set when they share an identifier; `null` when they share a title. */
data class Duplicate(val reference: Reference, val sharedIdentifier: String?)

/** What a merge moved from the removed reference into [kept]. */
data class MergeResult(
    val kept: Reference,
    val removedId: String,
    val fields: List<String>,
    val projects: Int,
    val tags: Int,
    val notes: Int,
    val attachments: Int,
    val highlights: Int,
    val relations: Int,
)

/** A reference that has issues, with them (worst first). */
data class CheckedReference(val id: String, val citeKey: String, val issues: List<ReferenceIssue>)

/** [kind]: `retraction`, `concern` (expression of concern) or another update (`correction`, …). */
data class Notice(val kind: String, val notice: String?, val date: String?, val source: String?)

data class RetractionResult(val reference: Reference, val notices: List<Notice>, val error: String?)
