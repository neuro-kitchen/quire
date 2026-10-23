package dev.neurokitchen.quire.presentation.screens.reader

import dev.neurokitchen.quire.presentation.screens.reader.components.ExtractedTable
import dev.neurokitchen.quire.presentation.screens.reader.components.ExtractedFigure
import dev.neurokitchen.quire.domain.repository.ReaderLayout
import dev.neurokitchen.quire.domain.model.PdfFigure
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
    val isAnnotationsCollapsed: Boolean = true,
    val isNotesCollapsed: Boolean = true,
    val readerTheme: ReaderTheme = ReaderTheme.Dark,
    val readerFont: ReaderFont = ReaderFont.Serif,
    val fontSizeSp: Int = 16,
    /** The engine's message when something failed. */
    val error: String? = null,
    val find: FindState = FindState(),
    val isFigureDrawerOpen: Boolean = false,
    val isSnapshotMode: Boolean = false,
    val activeCitation: dev.neurokitchen.quire.presentation.screens.reader.components.CitationDetail? = null,
    /** Pane sizes, from the preferences. */
    val layout: ReaderLayout = ReaderLayout(),
    /** A figure or table read beside the text or the PDF. */
    val pinned: Pinned? = null,
) {
    /** The main area's columns: the text and/or the PDF as the mode says, then what is pinned. */
    val panes: List<ReaderPane>
        get() {
            val hasPages = pdf.pages.isNotEmpty()
            val main = when {
                mode == ReaderMode.Split && hasPages -> listOf(ReaderPane.Text, ReaderPane.Pdf)
                mode == ReaderMode.Pdf && hasPages -> listOf(ReaderPane.Pdf)
                else -> listOf(ReaderPane.Text)
            }
            return if (pinned != null) main + ReaderPane.Pinned else main
        }
}

enum class ReaderPane { Text, Pdf, Pinned }

/** What can be pinned beside the text or the PDF. */
sealed interface Pinned {
    data class Figure(val figure: ExtractedFigure) : Pinned
    data class Table(val table: ExtractedTable) : Pinned
}

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
    /** Figures found from their captions; `null` until the figure drawer first opens. */
    val figures: List<PdfFigure>? = null,
)
