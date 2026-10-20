package dev.neurokitchen.quire.presentation.screens.library

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.SortColumn
import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import org.jetbrains.compose.resources.StringResource

@Immutable
data class LibraryState(
    val isLoading: Boolean = true,
    val query: String = "",
    val source: LibrarySource = LibrarySource.All,
    /** The project tree, flattened: only rows whose parents are expanded. */
    val projects: List<ProjectRowState> = emptyList(),
    /** The library's coloured tags, then the tags of the references in view; selected ones included. */
    val tags: List<TagState> = emptyList(),
    val sort: SortColumn = SortColumn.Title,
    val ascending: Boolean = true,
    val references: List<ReferenceCardState> = emptyList(),
    /** Selected reference IDs for single or multi-selection. */
    val selectedIds: Set<String> = emptySet(),
    /** The reference shown in the item pane (primary active item). */
    val selected: ReferenceDetailState? = null,
    val collapsedSections: Set<ItemSection> = emptySet(),
    /** The selected reference's fields being edited, if they are. */
    val editing: EditDraft? = null,
    /** What is typed in the Tags and Notes sections' inputs. */
    val tagDraft: String = "",
    val noteDraft: String = "",
    /** A page to open in the browser (a blocked PDF's landing page); the screen opens it once. */
    val pageToOpen: String? = null,
    val isSidebarCollapsed: Boolean = false,
    val isInspectorCollapsed: Boolean = false,
    /** The add dialog, when open. */
    val addDialog: AddDialogState? = null,
    /** The dialog asking for a name (new project, rename), when open. */
    val nameDialog: NameDialog? = null,
    /** An action waiting for the user's confirmation. */
    val confirm: Confirm? = null,
    /** What the last action did, or why it failed; shown until dismissed. */
    val notice: Notice? = null,
    /** The engine's message when the library could not be read. */
    val error: String? = null,
    /** Why the engine refused the search as typed (a filter half typed); the last results stay. */
    val searchError: String? = null,
    val savedSearches: List<SavedSearchState> = emptyList(),
    val isQuickLookOpen: Boolean = false,
)

/** A saved search; [isActive] when its query is the one in the search field. */
@Immutable
data class SavedSearchState(val name: String, val query: String, val isActive: Boolean)

/** The add dialog's tabs: by identifier (looked up), typed in, pasted BibTeX. */
enum class AddTab { Identifier, Manual, Bibtex }

@Immutable
data class AddDialogState(
    val tab: AddTab = AddTab.Identifier,
    /** DOI, arXiv ID, PubMed ID or URL. */
    val identifier: String = "",
    /** The looked-up work, before adding. */
    val preview: ReferenceCardState? = null,
    val manual: ManualEntry = ManualEntry(),
    val bibtex: String = "",
    /** A lookup or an add is running. */
    val isWorking: Boolean = false,
    /** The engine's message when the last lookup or add failed. */
    val error: String? = null,
) {
    val canAdd: Boolean
        get() = !isWorking && when (tab) {
            AddTab.Identifier -> identifier.isNotBlank()
            AddTab.Manual -> manual.title.isNotBlank()
            AddTab.Bibtex -> bibtex.isNotBlank()
        }
}

/** A reference typed in: authors as "Family, Given; Family, Given", tags comma-separated. */
@Immutable
data class ManualEntry(
    val title: String = "",
    val authors: String = "",
    val year: String = "",
    val containerTitle: String = "",
    val doi: String = "",
    val tags: String = "",
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

/** A tag in the tag selector; [color] `#rrggbb` for a coloured tag. */
@Immutable
data class TagState(val name: String, val isSelected: Boolean, val color: String? = null)

/** What a name is asked for. */
sealed interface NameDialog {
    /** A project, under [parentPath] if given, holding [withIds] (a selection) if given. */
    data class NewProject(val parentPath: String? = null, val withIds: Set<String> = emptySet()) : NameDialog
    data class RenameProject(val path: String, val current: String) : NameDialog
    data class RenameTag(val current: String) : NameDialog
    /** Saves [query] (the search field's) under a name. */
    data class SaveSearch(val query: String) : NameDialog
    data class RenameSavedSearch(val current: String) : NameDialog
}

/** An action to confirm first. */
sealed interface Confirm {
    /** The project and its subprojects; the references stay in the library. */
    data class DeleteProject(val path: String, val name: String) : Confirm
    /** The tag, from every reference. */
    data class DeleteTag(val name: String) : Confirm
}

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
    /** Project paths, parallel to [projects] (to take it out of one). */
    val projectPaths: List<String> = emptyList(),
    /** Loaded after selection: its notes, related references, PDF files and issues. */
    val notes: List<NoteState> = emptyList(),
    val related: List<RelatedState> = emptyList(),
    val pdfFiles: List<String> = emptyList(),
    val issues: List<String> = emptyList(),
    /** In the trash: the pane offers Restore instead of Delete. */
    val isTrashed: Boolean = false,
)

@Immutable
data class NoteState(val id: String, val body: String, val updated: String)

/** A related reference, to select or unrelate. */
@Immutable
data class RelatedState(val id: String, val title: String, val creatorYear: String)

/** The Info section being edited: the fields as text (authors "Family, Given; …"). */
@Immutable
data class EditDraft(
    val title: String,
    val authors: String,
    val date: String,
    val containerTitle: String,
    val volume: String,
    val issue: String,
    val pages: String,
    val doi: String,
    val abstract: String,
)

@Immutable
data class DetailField(val label: StringResource, val value: String)

/** The item pane's sections, in Zotero's order. */
enum class ItemSection { Info, Abstract, Attachments, Notes, Projects, Tags, Related }
