package dev.neurokitchen.quire.domain.model

/** A project (Zotero calls them collections), identified by its path (`thesis/chapter-2`). */
data class Project(
    val path: String,
    val name: String,
    val parentPath: String? = null,
    /** References in the project itself. */
    val referenceCount: Int = 0,
)
