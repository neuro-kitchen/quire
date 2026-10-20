package dev.neurokitchen.quire.presentation.screens.reader

import dev.neurokitchen.quire.domain.model.ReadingStatus

sealed interface ReaderEvent {
    data class OnModeChange(val mode: ReaderMode) : ReaderEvent
    data class OnTabClick(val index: Int) : ReaderEvent
    data object OnZoomIn : ReaderEvent
    data object OnZoomOut : ReaderEvent
    data object OnZoomReset : ReaderEvent
    /** Text selected in the text view: `prefix` is what comes just before it in its block. */
    data class OnHighlight(val quote: String, val prefix: String, val color: String = "yellow") : ReaderEvent
    data class OnHighlightClick(val id: String) : ReaderEvent
    data class OnHighlightComment(val id: String, val comment: String) : ReaderEvent
    data class OnHighlightRemove(val id: String) : ReaderEvent
    data class OnNoteDraftChange(val text: String) : ReaderEvent
    data object OnNoteAdd : ReaderEvent
    data class OnNoteRemove(val id: String) : ReaderEvent
    data class OnReadingChange(val status: ReadingStatus) : ReaderEvent
    data object OnToggleAnnotations : ReaderEvent
    data object OnToggleNotes : ReaderEvent
    data class OnSetReaderTheme(val theme: dev.neurokitchen.quire.presentation.designsystem.ReaderTheme) : ReaderEvent
    data class OnSetReaderFont(val font: dev.neurokitchen.quire.presentation.designsystem.ReaderFont) : ReaderEvent
    data class OnChangeFontSize(val delta: Int) : ReaderEvent
    /** Opens or closes the find bar. */
    data object OnFindToggle : ReaderEvent
    data class OnFindChange(val query: String) : ReaderEvent
    data object OnFindNext : ReaderEvent
    data object OnFindPrevious : ReaderEvent
}
