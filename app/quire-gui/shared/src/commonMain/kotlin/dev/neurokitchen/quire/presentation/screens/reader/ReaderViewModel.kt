package dev.neurokitchen.quire.presentation.screens.reader

import dev.neurokitchen.quire.presentation.platform.pageCacheBytes
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import dev.neurokitchen.quire.domain.repository.ReaderLayout
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import dev.neurokitchen.quire.domain.model.SelectedTextRange
import dev.neurokitchen.quire.domain.model.WordLocation
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.debounce
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.compose.ui.graphics.Canvas
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.Paint
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import dev.neurokitchen.quire.domain.model.PageRect
import dev.neurokitchen.quire.domain.model.PdfFigure
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.presentation.mapping.toImageBitmap
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.withContext
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** How long pane sizes stay unchanged before they are saved. */
private const val LAYOUT_SAVE_DELAY_MS = 500L

/**
 * One open paper: its PDF pages and text tabs, highlights (shown in both) and notes. Opens on the
 * PDF when there is one, else on the text.
 */
class ReaderViewModel(
    private val referenceId: String,
    title: String,
    private val reader: ReaderRepository,
    private val pdf: PdfRepository,
    /** Where rendered pixels become images (off the UI thread). */
    private val work: CoroutineDispatcher = Dispatchers.Default,
    /** Where pane sizes are kept; `null`: not kept. */
    private val preferences: PreferencesRepository? = null,
) : ViewModel(), PdfTextLayer {
    /** Rendered pages kept, in bytes of pixels: what the platform can spare (128 MB on the desktop). */
    private val pageCacheLimit = pageCacheBytes()

    private val _state = MutableStateFlow(
        ReaderState(referenceId = referenceId, title = title, layout = preferences?.preferences?.value?.readerLayout ?: ReaderLayout()),
    )
    val state: StateFlow<ReaderState> = _state.asStateFlow()

    /**
     * Rendered pages as images by (index, width step), most recently used last, within
     * [pageCacheLimit]. Widths are rounded up to [WIDTH_STEP], so resizing a little or zooming
     * back reuses a render; renders in progress are shared, not started twice.
     */
    private val pages = LinkedHashMap<Pair<Int, Int>, ImageBitmap>()
    private var pageBytes = 0L
    private val rendering = HashMap<Pair<Int, Int>, Deferred<ImageBitmap?>>()

    /** What is typed in the find bar; searched a moment after typing stops. */
    private val findQuery = MutableStateFlow("")

    init {
        // Pane sizes, saved once a drag has stopped for a moment (not on every move).
        preferences?.let { prefs ->
            viewModelScope.launch {
                _state.map { it.layout }.distinctUntilChanged().drop(1).debounce(LAYOUT_SAVE_DELAY_MS).collect { prefs.setReaderLayout(it) }
            }
        }
        viewModelScope.launch {
            findQuery.debounce(FIND_DELAY_MS).collectLatest { query -> find(query) }
        }
        load()
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
            is ReaderEvent.OnModeChange -> _state.update { it.copy(mode = event.mode, error = null, pdf = it.pdf.copy(error = null)) }
            is ReaderEvent.OnTabClick -> {
                _state.update { it.copy(selectedTab = event.index.coerceIn(0, (it.tabs.size - 1).coerceAtLeast(0)), error = null) }
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
            is ReaderEvent.OnSetZoom -> zoom { event.zoom }
            is ReaderEvent.OnHighlight -> act {
                val quote = event.quote.replace(Regex("\\s+"), " ").trim()
                val prefix = event.prefix.replace(Regex("\\s+"), " ").trim()
                if (quote.isNotEmpty()) reader.addHighlight(referenceId, quote, prefix.takeLast(PREFIX_LENGTH), color = event.color)
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
            ReaderEvent.OnToggleFigureDrawer -> {
                _state.update { it.copy(isFigureDrawerOpen = !it.isFigureDrawerOpen) }
                if (_state.value.isFigureDrawerOpen) loadPdfFigures()
            }
            ReaderEvent.OnToggleSnapshotMode -> _state.update { it.copy(isSnapshotMode = !it.isSnapshotMode) }
            is ReaderEvent.OnShowCitation -> _state.update { it.copy(activeCitation = event.citation) }
            is ReaderEvent.OnLayoutChange -> _state.update { it.copy(layout = event.layout) }
            is ReaderEvent.OnPin -> _state.update { it.copy(pinned = event.pinned) }
        }
    }

    /**
     * Page `index` of the PDF, at least `width` pixels wide (drawn scaled to fit); the next pages
     * are rendered ahead, so scrolling finds them ready.
     */
    suspend fun page(index: Int, width: Int): ImageBitmap? {
        val image = render(index, width)
        val count = _state.value.pdf.pages.size
        for (next in index + 1..minOf(index + AHEAD, count - 1)) {
            viewModelScope.launch { render(next, width) }
        }
        return image
    }

    /** Looks for the PDF's figures once (it renders the pages with captions). */
    private fun loadPdfFigures() {
        val state = _state.value
        val path = state.pdf.path ?: return
        if (state.pdf.figures != null || figuresLoading) return
        figuresLoading = true
        viewModelScope.launch {
            val found = runCatching { pdf.figures(path) }.getOrDefault(emptyList())
            _state.update { it.copy(pdf = it.pdf.copy(figures = found)) }
            figuresLoading = false
        }
    }

    private var figuresLoading = false
    private val figureCrops = HashMap<PdfFigure, ImageBitmap>()

    /** `figure` cut from its page, rendered [FIGURE_PAGE_WIDTH] pixels wide. */
    suspend fun pdfFigure(figure: PdfFigure): ImageBitmap? {
        figureCrops[figure]?.let { return it }
        val size = _state.value.pdf.pages.getOrNull(figure.page) ?: return null
        val page = render(figure.page, FIGURE_PAGE_WIDTH) ?: return null
        val crop = withContext(work) { crop(page, figure.rect, page.width / size.width) } ?: return null
        figureCrops[figure] = crop
        return crop
    }

    /** The rendered page `index` nearest to `width`, if any: shown while a sharp one renders. */
    fun renderedNear(index: Int, width: Int): ImageBitmap? =
        pages.entries.filter { it.key.first == index }.minByOrNull { kotlin.math.abs(it.key.second - width) }?.value

    private suspend fun render(index: Int, width: Int): ImageBitmap? {
        val path = _state.value.pdf.path ?: return null
        val key = index to stepped(width)
        pages.remove(key)?.let { pages[key] = it; return it }
        val job = rendering.getOrPut(key) {
            viewModelScope.async {
                runCatching { withContext(work) { pdf.render(path, index, key.second).toImageBitmap() } }
                    .onFailure { e ->
                        if (e !is kotlinx.coroutines.CancellationException && !e.message.orEmpty().contains("left the composition")) {
                            _state.update { it.copy(pdf = it.pdf.copy(error = e.message)) }
                        }
                    }
                    .getOrNull()
                    ?.also { keep(key, it) }
                    .also { rendering.remove(key) }
            }
        }
        return job.await()
    }

    private fun keep(key: Pair<Int, Int>, image: ImageBitmap) {
        pages.remove(key)?.let { pageBytes -= it.bytes }
        pages[key] = image
        pageBytes += image.bytes
        // Least recently used first (insertion order is common stdlib; an access-ordered map is JVM-only).
        while (pageBytes > pageCacheLimit && pages.size > 1) {
            val oldest = pages.keys.first()
            pages.remove(oldest)?.let { pageBytes -= it.bytes }
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

    /** The PDF, the text and the reading status load at once: none waits for another. */
    private fun load() {
        viewModelScope.launch {
            runCatching { reader.opened(referenceId) }.onSuccess { status -> _state.update { it.copy(reading = status) } }
        }
        viewModelScope.launch { loadPdf() }
        viewModelScope.launch { loadText() }
    }

    private suspend fun loadPdf() {
        val path = runCatching { reader.pdfFiles(referenceId).firstOrNull() }.getOrNull()
        if (path == null) {
            _state.update { it.copy(pdf = it.pdf.copy(isLoading = false)) }
            return
        }
        runCatching { pdf.pageSizes(path) }
            .onSuccess { sizes ->
                _state.update { it.copy(mode = ReaderMode.Pdf, pdf = it.pdf.copy(path = path, pages = sizes, isLoading = false)) }
            }
            .onFailure { e -> _state.update { it.copy(pdf = it.pdf.copy(error = e.message ?: e.toString(), isLoading = false)) } }
        // The pages are on screen already; the highlights' places follow.
        locateHighlights()
    }

    private suspend fun loadText() {
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

    private fun fail(e: Throwable) {
        if (e is kotlinx.coroutines.CancellationException || e.message.orEmpty().contains("left the composition")) return
        _state.update { it.copy(error = e.message ?: e.toString()) }
    }

    private companion object {
        /** Pages rendered ahead of the one asked for. */
        const val AHEAD = 2
        /** Width of the pages figures are cut from (a width step: rendered as asked). */
        const val FIGURE_PAGE_WIDTH = 1536
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

/** Widths of rendered pages go in steps of this many pixels. */
private const val WIDTH_STEP = 256

/** `width` rounded up to the next [WIDTH_STEP]. */
private fun stepped(width: Int): Int = ((width.coerceAtLeast(1) + WIDTH_STEP - 1) / WIDTH_STEP) * WIDTH_STEP

/** The part `rect` (points) of `page`, `scale` pixels per point; `null` when nothing is left. */
private fun crop(page: ImageBitmap, rect: PageRect, scale: Float): ImageBitmap? {
    val x = (rect.left * scale).toInt().coerceIn(0, page.width - 1)
    val y = (rect.top * scale).toInt().coerceIn(0, page.height - 1)
    val w = ((rect.right - rect.left) * scale).toInt().coerceAtMost(page.width - x)
    val h = ((rect.bottom - rect.top) * scale).toInt().coerceAtMost(page.height - y)
    if (w <= 0 || h <= 0) return null
    val out = ImageBitmap(w, h)
    Canvas(out).drawImageRect(page, IntOffset(x, y), IntSize(w, h), IntOffset.Zero, IntSize(w, h), Paint())
    return out
}

/** Bytes of pixels the image holds (RGBA). */
private val ImageBitmap.bytes: Long get() = width.toLong() * height * 4
