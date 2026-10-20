package dev.neurokitchen.quire.presentation.screens.library

import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.SortColumn

sealed interface LibraryEvent {
    data class OnQueryChange(val query: String) : LibraryEvent
    data class OnSourceClick(val source: LibrarySource) : LibraryEvent
    data class OnProjectExpandToggle(val id: String) : LibraryEvent
    data class OnTagClick(val name: String) : LibraryEvent
    /** Same column again flips the direction. */
    data class OnSortClick(val column: SortColumn) : LibraryEvent
    data class OnReferenceClick(val id: String) : LibraryEvent
    data class OnReferenceSelect(val id: String, val isCtrlPressed: Boolean = false, val isShiftPressed: Boolean = false) : LibraryEvent
    data object OnSelectAll : LibraryEvent
    data object OnClearSelection : LibraryEvent
    data object OnCloseDetail : LibraryEvent
    data class OnSectionToggle(val section: ItemSection) : LibraryEvent
    data class OnAddTabSelect(val tab: AddTab) : LibraryEvent
    data class OnIdentifierChange(val identifier: String) : LibraryEvent
    /** Looks the identifier up to preview it. */
    data object OnLookup : LibraryEvent
    data class OnManualChange(val entry: ManualEntry) : LibraryEvent
    data class OnBibtexChange(val text: String) : LibraryEvent
    /** Adds what the dialog's tab holds, into the project in view if one is. */
    data object OnAddConfirm : LibraryEvent
    data object OnDismissNotice : LibraryEvent

    // The item pane, on the selected reference.
    data object OnEditStart : LibraryEvent
    data class OnEditChange(val draft: EditDraft) : LibraryEvent
    data object OnEditSave : LibraryEvent
    data object OnEditCancel : LibraryEvent
    data class OnTagDraftChange(val text: String) : LibraryEvent
    /** Adds the tag draft's comma-separated names. */
    data object OnTagAdd : LibraryEvent
    data class OnTagRemove(val name: String) : LibraryEvent
    data class OnProjectRemove(val path: String) : LibraryEvent
    data class OnUnrelate(val id: String) : LibraryEvent
    data class OnNoteDraftChange(val text: String) : LibraryEvent
    data object OnNoteAdd : LibraryEvent
    data class OnNoteDelete(val id: String) : LibraryEvent
    data object OnGetPdf : LibraryEvent
    /** Attaches the file at `path` (picked by the user). */
    data class OnAttachPdf(val path: String) : LibraryEvent
    data object OnRemovePdfs : LibraryEvent
    /** Opens the publisher's page in the browser (for a PDF the site blocks). */
    data object OnOpenPublisherPage : LibraryEvent
    data object OnPageOpened : LibraryEvent
    data object OnRefetch : LibraryEvent
    data object OnRestore : LibraryEvent
    data class OnDeleteReference(val id: String) : LibraryEvent
    data object OnDeleteSelected : LibraryEvent
    data class OnBatchSetReadingStatus(val ids: Set<String>, val status: dev.neurokitchen.quire.domain.model.ReadingStatus) : LibraryEvent
    data class OnBatchAddToProject(val ids: Set<String>, val projectPath: String) : LibraryEvent
    data class OnCreateCollectionFromSelection(val name: String, val ids: Set<String>) : LibraryEvent
    data class OnCreateProject(val name: String, val parentPath: String? = null) : LibraryEvent
    data class OnShowAddDialog(val show: Boolean) : LibraryEvent
    /** Opens a name dialog, or closes it (`null`). */
    data class OnNameDialog(val dialog: NameDialog?) : LibraryEvent
    /** The name typed in the open name dialog. */
    data class OnNameConfirm(val name: String) : LibraryEvent
    /** Asks to confirm an action, or cancels (`null`). */
    data class OnConfirm(val confirm: Confirm?) : LibraryEvent
    /** The user confirmed the pending action. */
    data object OnConfirmed : LibraryEvent
    /** Runs a saved search: its query goes in the search field, over the whole library. */
    data class OnSavedSearchClick(val name: String) : LibraryEvent
    data class OnSavedSearchRemove(val name: String) : LibraryEvent
    /** Adds the paper the PDF at `path` describes, with the PDF, into the project in view. */
    data class OnImportPdf(val path: String) : LibraryEvent
    /** Colours a tag (`#rrggbb`), or takes its colour away (`null`). */
    data class OnTagColor(val name: String, val color: String?) : LibraryEvent
    data object OnToggleSidebar : LibraryEvent
    data object OnToggleInspector : LibraryEvent
}
