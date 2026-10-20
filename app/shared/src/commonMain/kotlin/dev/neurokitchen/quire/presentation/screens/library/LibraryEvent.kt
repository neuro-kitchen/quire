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
    data object OnCloseDetail : LibraryEvent
    data class OnSectionToggle(val section: ItemSection) : LibraryEvent
}
