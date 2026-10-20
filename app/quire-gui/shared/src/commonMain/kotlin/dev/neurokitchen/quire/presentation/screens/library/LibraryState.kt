package dev.neurokitchen.quire.presentation.screens.library

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.SortColumn
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import org.jetbrains.compose.resources.StringResource

@Immutable
data class LibraryState(
    val isLoading: Boolean = true,
    val query: String = "",
    val source: LibrarySource = LibrarySource.All,
    /** The project tree, flattened: only rows whose parents are expanded. */
    val projects: List<ProjectRowState> = emptyList(),
    /** Tags of the references in view, selected ones included. */
    val tags: List<TagState> = emptyList(),
    val sort: SortColumn = SortColumn.Title,
    val ascending: Boolean = true,
    val references: List<ReferenceCardState> = emptyList(),
    /** Selected reference IDs for single or multi-selection. */
    val selectedIds: Set<String> = emptySet(),
    /** The reference shown in the item pane (primary active item). */
    val selected: ReferenceDetailState? = null,
    val collapsedSections: Set<ItemSection> = emptySet(),
    val isSidebarCollapsed: Boolean = false,
    val isInspectorCollapsed: Boolean = false,
    val showAddDialog: Boolean = false,
    val showAddProjectDialog: Boolean = false,
    /** The engine's message when the library could not be read. */
    val error: String? = null,
)

@Immutable
data class ProjectRowState(
    /** The project's path, `thesis/chapter-2`. */
    val id: String,
    val name: String,
    val depth: Int,
    val hasChildren: Boolean,
    val isExpanded: Boolean,
    val isSelected: Boolean,
    val referenceCount: Int = 0,
)

@Immutable
data class TagState(val name: String, val isSelected: Boolean)

@Immutable
data class ReferenceDetailState(
    val id: String,
    val title: String,
    val citeKey: String,
    /** CSL type. */
    val kind: String,
    val creator: String,
    val year: String,
    /** "Family, Given" in order. */
    val authors: List<String>,
    /** The Info section's other rows, empty values left out. */
    val fields: List<DetailField>,
    val abstract: String,
    val tags: List<String>,
    /** Project names. */
    val projects: List<String>,
    val hasPdf: Boolean,
    val reading: ReadingStatus,
)

@Immutable
data class DetailField(val label: StringResource, val value: String)

/** The item pane's sections, in Zotero's order. */
enum class ItemSection { Info, Abstract, Attachments, Notes, Projects, Tags, Related }
