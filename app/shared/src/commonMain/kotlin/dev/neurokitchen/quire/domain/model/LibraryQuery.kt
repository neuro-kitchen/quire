package dev.neurokitchen.quire.domain.model

/** Which references to list and in what order; the engine answers it. */
data class LibraryQuery(
    val source: LibrarySource = LibrarySource.All,
    /** Search words (the engine's full-text search). */
    val text: String = "",
    /** References must carry every one of these tags. */
    val tags: Set<String> = emptySet(),
    val sort: SortColumn = SortColumn.Title,
    val ascending: Boolean = true,
)

sealed interface LibrarySource {
    data object All : LibrarySource
    data object Unfiled : LibrarySource
    data object Trash : LibrarySource
    data class InProject(val path: String) : LibrarySource
}

enum class SortColumn { Title, Creator, Year, DateAdded }
