package dev.neurokitchen.quire.presentation.screens.reader

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * One open paper: its PDF pages and text tabs, highlights (shown in both) and notes. Opens on the
 * PDF when there is one, else on the text.
 */
class ReaderViewModel(
    private val referenceId: String,
    title: String,
    private val reader: ReaderRepository,
    private val pdf: PdfRepository,
) : ViewModel() {
    private val _state = MutableStateFlow(ReaderState(referenceId = referenceId, title = title))
    val state: StateFlow<ReaderState> = _state.asStateFlow()

    /** Rendered pages by (index, width), most recent last: the PDF view asks as pages scroll in. */
    private val pages = LinkedHashMap<Pair<Int, Int>, PageImage>()

    init {
        viewModelScope.launch { load() }
        viewModelScope.launch {
            reader.highlights(referenceId)
                .catch { e -> fail(e) }
                .collect { highlights ->
                    _state.update { it.copy(highlights = highlights) }
                    locateHighlights()
                }
        }
        viewModelScope.launch {
            reader.notes(referenceId).catch { e -> fail(e) }.collect { notes -> _state.update { it.copy(notes = notes) } }
        }
    }

    fun onEvent(event: ReaderEvent) {
        when (event) {
            is ReaderEvent.OnModeChange -> _state.update { it.copy(mode = event.mode) }
            is ReaderEvent.OnTabClick -> _state.update { it.copy(selectedTab = event.index.coerceIn(0, (it.tabs.size - 1).coerceAtLeast(0))) }
            ReaderEvent.OnZoomIn -> zoom { it * ZOOM_STEP }
            ReaderEvent.OnZoomOut -> zoom { it / ZOOM_STEP }
            ReaderEvent.OnZoomReset -> zoom { 1f }
            is ReaderEvent.OnHighlight -> act {
                val quote = event.quote.trim()
                if (quote.isNotEmpty()) reader.addHighlight(referenceId, quote, event.prefix.takeLast(PREFIX_LENGTH), color = event.color)
            }
            is ReaderEvent.OnHighlightClick -> _state.update { it.copy(focusedHighlight = event.id) }
            is ReaderEvent.OnHighlightComment -> act { reader.commentHighlight(event.id, event.comment) }
            is ReaderEvent.OnHighlightRemove -> act { reader.removeHighlight(event.id) }
            is ReaderEvent.OnNoteDraftChange -> _state.update { it.copy(noteDraft = event.text) }
            ReaderEvent.OnNoteAdd -> {
                val body = _state.value.noteDraft.trim()
                if (body.isNotEmpty()) act {
                    reader.addNote(referenceId, body)
                    _state.update { it.copy(noteDraft = "") }
                }
            }
            is ReaderEvent.OnNoteRemove -> act { reader.removeNote(event.id) }
            is ReaderEvent.OnReadingChange -> act {
                reader.setReading(referenceId, event.status)
                _state.update { it.copy(reading = event.status) }
            }
            ReaderEvent.OnToggleAnnotations -> _state.update { it.copy(isAnnotationsCollapsed = !it.isAnnotationsCollapsed) }
            ReaderEvent.OnToggleNotes -> _state.update { it.copy(isNotesCollapsed = !it.isNotesCollapsed) }
            is ReaderEvent.OnSetReaderTheme -> _state.update { it.copy(readerTheme = event.theme) }
            is ReaderEvent.OnSetReaderFont -> _state.update { it.copy(readerFont = event.font) }
            is ReaderEvent.OnChangeFontSize -> _state.update {
                it.copy(fontSizeSp = (it.fontSizeSp + event.delta).coerceIn(12, 28))
            }
        }
    }

    /** Page `index` of the PDF, `width` pixels wide (cached). */
    suspend fun page(index: Int, width: Int): PageImage? {
        val path = _state.value.pdf.path ?: return null
        val key = index to width
        pages.remove(key)?.let { pages[key] = it; return it }
        return runCatching { pdf.render(path, index, width) }
            .onFailure { e -> _state.update { it.copy(pdf = it.pdf.copy(error = e.message)) } }
            .getOrNull()
            ?.also {
                pages[key] = it
                // Insertion order is common stdlib (an access-ordered map is JVM-only).
                while (pages.size > PAGE_CACHE) pages.remove(pages.keys.first())
            }
    }

    /** The local file of a figure image, `null` if it cannot be had. */
    suspend fun figure(url: String): String? = runCatching { reader.figure(url) }.getOrNull()

    private suspend fun load() {
        runCatching { reader.opened(referenceId) }.onSuccess { status -> _state.update { it.copy(reading = status) } }
        val path = runCatching { reader.pdfFiles(referenceId).firstOrNull() }.getOrNull()
        if (path != null) {
            runCatching { pdf.pageSizes(path) }
                .onSuccess { sizes -> _state.update { it.copy(mode = ReaderMode.Pdf, pdf = it.pdf.copy(path = path, pages = sizes)) } }
                .onFailure { e -> _state.update { it.copy(pdf = it.pdf.copy(error = e.message ?: e.toString())) } }
            locateHighlights()
        }
        runCatching { reader.paper(referenceId) }
            .onSuccess { paper ->
                _state.update {
                    it.copy(isLoading = false, tabs = paper.tabs, source = paper.source, textError = paper.textError,
                        // Open on the paper's text rather than its abstract when there is text.
                        selectedTab = if (paper.tabs.size > 1) 1 else 0)
                }
            }
            .onFailure { e -> _state.update { it.copy(isLoading = false) }; fail(e) }
    }

    private suspend fun locateHighlights() {
        val state = _state.value
        val path = state.pdf.path ?: return
        if (state.highlights.isEmpty()) {
            _state.update { it.copy(pdf = it.pdf.copy(locations = emptyList())) }
            return
        }
        runCatching { pdf.locate(path, state.highlights) }
            .onSuccess { locations -> _state.update { it.copy(pdf = it.pdf.copy(locations = locations)) } }
    }

    private fun zoom(change: (Float) -> Float) = _state.update {
        it.copy(pdf = it.pdf.copy(zoom = change(it.pdf.zoom).coerceIn(MIN_ZOOM, MAX_ZOOM)))
    }

    private fun act(block: suspend () -> Unit) = viewModelScope.launch {
        runCatching { block() }.onFailure { fail(it) }
    }

    private fun fail(e: Throwable) = _state.update { it.copy(error = e.message ?: e.toString()) }

    private companion object {
        const val PAGE_CACHE = 12
        const val ZOOM_STEP = 1.25f
        const val MIN_ZOOM = 0.5f
        const val MAX_ZOOM = 4f
        /** Characters kept before a quote to tell repeated quotes apart. */
        const val PREFIX_LENGTH = 40
    }
}
