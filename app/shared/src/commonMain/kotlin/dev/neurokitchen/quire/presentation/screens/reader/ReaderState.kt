package dev.neurokitchen.quire.presentation.screens.reader

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.PaperTab
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.domain.model.ReadingStatus

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
    /** The engine's message when something failed. */
    val error: String? = null,
)

enum class ReaderMode { Pdf, Text }

@Immutable
data class PdfViewState(
    /** The first attached PDF; `null` without one. */
    val path: String? = null,
    val pages: List<PageSize> = emptyList(),
    /** 1 = fit the pane's width. */
    val zoom: Float = 1f,
    val locations: List<QuoteLocation> = emptyList(),
    val error: String? = null,
)
