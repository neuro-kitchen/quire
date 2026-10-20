package dev.neurokitchen.quire.presentation.screens.reader

import dev.neurokitchen.quire.domain.model.ReadingStatus

sealed interface ReaderEvent {
    data class OnModeChange(val mode: ReaderMode) : ReaderEvent
    data class OnTabClick(val index: Int) : ReaderEvent
    data object OnZoomIn : ReaderEvent
    data object OnZoomOut : ReaderEvent
    data object OnZoomReset : ReaderEvent
    /** Text selected in the text view: `prefix` is what comes just before it in its block. */
    data class OnHighlight(val quote: String, val prefix: String) : ReaderEvent
    data class OnHighlightClick(val id: String) : ReaderEvent
    data class OnHighlightComment(val id: String, val comment: String) : ReaderEvent
    data class OnHighlightRemove(val id: String) : ReaderEvent
    data class OnNoteDraftChange(val text: String) : ReaderEvent
    data object OnNoteAdd : ReaderEvent
    data class OnNoteRemove(val id: String) : ReaderEvent
    data class OnReadingChange(val status: ReadingStatus) : ReaderEvent
}
