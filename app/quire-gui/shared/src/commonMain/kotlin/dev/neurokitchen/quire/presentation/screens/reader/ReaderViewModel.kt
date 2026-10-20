package dev.neurokitchen.quire.presentation.screens.reader

import dev.neurokitchen.quire.domain.model.SelectedTextRange
import dev.neurokitchen.quire.domain.model.WordLocation
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.debounce
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
) : ViewModel(), PdfTextLayer {
    private val _state = MutableStateFlow(ReaderState(referenceId = referenceId, title = title))
    val state: StateFlow<ReaderState> = _state.asStateFlow()

    /** Rendered pages by (index, width), most recent last: the PDF view asks as pages scroll in. */
    private val pages = LinkedHashMap<Pair<Int, Int>, PageImage>()

    /** What is typed in the find bar; searched a moment after typing stops. */
    private val findQuery = MutableStateFlow("")

    init {
        viewModelScope.launch {
            findQuery.debounce(FIND_DELAY_MS).collectLatest { query -> find(query) }
        }
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
            is ReaderEvent.OnTabClick -> {
                _state.update { it.copy(selectedTab = event.index.coerceIn(0, (it.tabs.size - 1).coerceAtLeast(0))) }
                _state.update { it.copy(find = it.find.copy(textMatches = textMatches(it, it.find.query), current = 0)) }
            }
            ReaderEvent.OnFindToggle -> _state.update { it.copy(find = it.find.copy(isOpen = !it.find.isOpen)) }
            is ReaderEvent.OnFindChange -> {
                _state.update { it.copy(find = it.find.copy(query = event.query)) }
                findQuery.value = event.query
            }
            ReaderEvent.OnFindNext -> step(1)
            ReaderEvent.OnFindPrevious -> step(-1)
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

    override suspend fun glyphAt(page: Int, x: Float, y: Float): Int? =
        _state.value.pdf.path?.let { path -> runCatching { pdf.nearestGlyph(path, page, x, y) }.getOrNull() }

    override suspend fun range(page: Int, start: Int, end: Int): SelectedTextRange? =
        _state.value.pdf.path?.let { path -> runCatching { pdf.textRange(path, page, start, end) }.getOrNull() }

    override suspend fun wordAt(page: Int, x: Float, y: Float): WordLocation? =
        _state.value.pdf.path?.let { path -> runCatching { pdf.wordAt(path, page, x, y) }.getOrNull() }

    /** Matches on the PDF's pages (the engine) and in the text tab shown, for `query`. */
    private suspend fun find(query: String) {
        val path = _state.value.pdf.path
        val pdfMatches = if (query.isBlank() || path == null) emptyList()
            else runCatching { pdf.search(path, query) }.onFailure { fail(it) }.getOrDefault(emptyList())
        _state.update { it.copy(find = it.find.copy(pdfMatches = pdfMatches, textMatches = textMatches(it, query), current = 0)) }
    }

    /** Moves to the next (`1`) or previous (`-1`) match, around the ends. */
    private fun step(by: Int) = _state.update {
        val count = it.find.count(it.mode)
        if (count == 0) it else it.copy(find = it.find.copy(current = (it.find.current + by).mod(count)))
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
        const val FIND_DELAY_MS = 150L
    }
}

/** Every case-insensitive occurrence of `query` in the blocks of the text tab shown. */
private fun textMatches(state: ReaderState, query: String): List<TextMatch> {
    val needle = query.trim()
    if (needle.isEmpty()) return emptyList()
    val blocks = state.tabs.getOrNull(state.selectedTab)?.blocks.orEmpty()
    return blocks.flatMapIndexed { index, block ->
        val text = block.text
        generateSequence(text.indexOf(needle, ignoreCase = true).takeIf { it >= 0 }) { from ->
            text.indexOf(needle, from + needle.length, ignoreCase = true).takeIf { it >= 0 }
        }.map { TextMatch(index, it, it + needle.length) }.toList()
    }
}
