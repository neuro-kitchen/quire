package dev.neurokitchen.quire.presentation.screens.reader

import dev.neurokitchen.quire.domain.model.PdfMatch
import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.PaperTab
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme

@Immutable
data class ReaderState(
    val referenceId: String,
    val title: String,
    val isLoading: Boolean = true,
    val mode: ReaderMode = ReaderMode.Text,
    val pdf: PdfViewState = PdfViewState(),
    /** The abstract tab, then the paper's text tabs. */
    val tabs: List<PaperTab> = emptyList(),
    val selectedTab: Int = 0,
    /** Where the text came from, or why there is none. */
    val source: String? = null,
    val textError: String? = null,
    val highlights: List<Highlight> = emptyList(),
    /** The highlight to bring into view. */
    val focusedHighlight: String? = null,
    val notes: List<Note> = emptyList(),
    val noteDraft: String = "",
    val reading: ReadingStatus = ReadingStatus.Unread,
    val isAnnotationsCollapsed: Boolean = false,
    val isNotesCollapsed: Boolean = false,
    val readerTheme: ReaderTheme = ReaderTheme.Dark,
    val readerFont: ReaderFont = ReaderFont.Serif,
    val fontSizeSp: Int = 16,
    /** The engine's message when something failed. */
    val error: String? = null,
    val find: FindState = FindState(),
    val isFigureDrawerOpen: Boolean = false,
    val isSnapshotMode: Boolean = false,
    val activeCitation: dev.neurokitchen.quire.presentation.screens.reader.components.CitationDetail? = null,
)

/**
 * Find in the paper: on the PDF's pages ([pdfMatches], through the engine) or in the text tab shown
 * ([textMatches]); [current] indexes the list of the mode shown.
 */
@Immutable
data class FindState(
    val isOpen: Boolean = false,
    val query: String = "",
    val pdfMatches: List<PdfMatch> = emptyList(),
    val textMatches: List<TextMatch> = emptyList(),
    val current: Int = 0,
) {
    fun count(mode: ReaderMode) = if (mode == ReaderMode.Pdf) pdfMatches.size else textMatches.size
}

/** A match in a text tab: block [block], characters [start] to [end] of its text. */
@Immutable
data class TextMatch(val block: Int, val start: Int, val end: Int)

enum class ReaderMode { Pdf, Text, Split }

@Immutable
data class PdfViewState(
    /** The first attached PDF; `null` without one. */
    val path: String? = null,
    val pages: List<PageSize> = emptyList(),
    /** 1 = fit the pane's width. */
    val zoom: Float = 1f,
    val locations: List<QuoteLocation> = emptyList(),
    val error: String? = null,
    /** Still finding the PDF and its pages (the text loads on its own). */
    val isLoading: Boolean = true,
)
