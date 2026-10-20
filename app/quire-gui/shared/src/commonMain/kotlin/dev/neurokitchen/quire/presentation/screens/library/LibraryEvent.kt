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
    data class OnAddReference(val reference: dev.neurokitchen.quire.domain.model.Reference) : LibraryEvent
    data class OnDeleteReference(val id: String) : LibraryEvent
    data object OnDeleteSelected : LibraryEvent
    data class OnBatchSetReadingStatus(val ids: Set<String>, val status: dev.neurokitchen.quire.domain.model.ReadingStatus) : LibraryEvent
    data class OnBatchAddToProject(val ids: Set<String>, val projectPath: String) : LibraryEvent
    data class OnCreateCollectionFromSelection(val name: String, val ids: Set<String>) : LibraryEvent
    data class OnCreateProject(val name: String, val parentPath: String? = null) : LibraryEvent
    data class OnShowAddDialog(val show: Boolean) : LibraryEvent
    data class OnShowAddProjectDialog(val show: Boolean) : LibraryEvent
    data object OnToggleSidebar : LibraryEvent
    data object OnToggleInspector : LibraryEvent
}
