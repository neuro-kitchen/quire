package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.ui.input.pointer.PointerInputChange
import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.ui.unit.max
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.min
import dev.neurokitchen.quire.domain.model.PageRect
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.PdfMatch
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import dev.neurokitchen.quire.presentation.designsystem.readerColors
import dev.neurokitchen.quire.presentation.screens.reader.PdfTextLayer
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.reader_page_of
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.stringResource
import kotlin.math.max
import kotlin.math.roundToInt

/** Text selected on a page: glyphs [start] until [end], with its text and line boxes. */
data class ActivePdfSelection(
    val page: Int,
    val start: Int,
    val end: Int,
    val rects: List<PageRect>,
    val text: String,
)

/**
 * The PDF's pages in a scrolling list, with highlights and find matches drawn over them. Dragging
 * selects text through the [textLayer] (a double-click selects a word); the palette highlights,
 * notes or copies the selection.
 */
@Composable
fun PdfPagesView(
    pages: List<PageSize>,
    zoom: Float,
    locations: List<QuoteLocation>,
    focused: String?,
    loadPage: suspend (index: Int, widthPx: Int) -> ImageBitmap?,
    modifier: Modifier = Modifier,
    /** A page already rendered at another width: shown at once, until the sharp one is ready. */
    renderedNear: (index: Int, widthPx: Int) -> ImageBitmap? = { _, _ -> null },
    readerTheme: ReaderTheme = ReaderTheme.Dark,
    textLayer: PdfTextLayer = PdfTextLayer.None,
    matches: List<PdfMatch> = emptyList(),
    /** The match to bring into view and mark; ignored without matches. */
    currentMatch: Int = 0,
    isSnapshotMode: Boolean = false,
    onHighlight: (quote: String, prefix: String, color: String) -> Unit = { _, _, _ -> },
    onAddNote: (body: String) -> Unit = {},
    onAddSnapshot: (rect: PageRect, pageIndex: Int) -> Unit = { _, _ -> },
    onProgressChange: (page: Int, total: Int) -> Unit = { _, _ -> },
    onTapPage: () -> Unit = {},
    /** Room for bars drawn over the pages (the phone's frosted bars): the pages scroll under them. */
    contentPadding: PaddingValues = PaddingValues(),
    /** A page to bring to the top (the phone's page slider); `null`: none. */
    jumpToPage: Int? = null,
    onJumped: () -> Unit = {},
    /** A pinch ended at this zoom; `null`: pinching does nothing (the desktop zooms with keys). */
    onPinchZoom: ((Float) -> Unit)? = null,
    /**
     * Touch screens: text is selected after a long-press, so a plain drag scrolls (up, down and
     * sideways when zoomed). With a mouse a drag selects.
     */
    selectAfterLongPress: Boolean = false,
) {
    val list = rememberLazyListState()
    val scope = rememberCoroutineScope()
    val clipboard = LocalClipboardManager.current
    val focusedPage = locations.firstOrNull { it.highlightId == focused }?.page
    LaunchedEffect(focusedPage) { if (focusedPage != null) list.animateScrollToItem(focusedPage) }
    val current = matches.getOrNull(currentMatch)
    LaunchedEffect(current) { if (current != null) list.animateScrollToItem(current.page) }
    LaunchedEffect(jumpToPage) {
        if (jumpToPage != null) {
            list.scrollToItem(jumpToPage)
            onJumped()
        }
    }

    var selection by remember { mutableStateOf<ActivePdfSelection?>(null) }
    val firstVisiblePage by remember(pages.size) {
        derivedStateOf {
            if (!list.canScrollForward && (list.firstVisibleItemIndex > 0 || list.firstVisibleItemScrollOffset > 0)) {
                pages.size.coerceAtLeast(1)
            } else {
                list.firstVisibleItemIndex + 1
            }
        }
    }
    val themeColors = readerColors(readerTheme)

    LaunchedEffect(firstVisiblePage, pages.size) {
        onProgressChange(firstVisiblePage, pages.size)
    }

    // A pinch scales the pages as they are while the fingers move (no render on every frame);
    // the zoom is set when they lift, and the pages render once at the new width.
    var pinch by remember { mutableFloatStateOf(1f) }
    val horizontal = rememberScrollState()

    BoxWithConstraints(
        modifier
            .fillMaxSize()
            .background(themeColors.background)
            .then(
                if (onPinchZoom == null) Modifier else Modifier.pinchToZoom(
                    onPinch = { pinch = (pinch * it).coerceIn(0.5f / zoom, 4f / zoom) },
                    onEnd = {
                        if (pinch != 1f) onPinchZoom(zoom * pinch)
                        pinch = 1f
                    },
                ),
            ),
    ) {
        val pageWidth = min(maxWidth - Dimens.SpaceLg * 2, MaxPageWidth) * zoom
        // Zoomed in, pages are wider than the view: it scrolls sideways too.
        val contentWidth = max(maxWidth, pageWidth + Dimens.SpaceMd * 2)

        LazyColumn(
            state = list,
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceMd),
            contentPadding = PaddingValues(
                start = Dimens.SpaceMd,
                end = Dimens.SpaceMd,
                top = Dimens.SpaceLg + contentPadding.calculateTopPadding(),
                bottom = Dimens.SpaceLg + contentPadding.calculateBottomPadding(),
            ),
            modifier = Modifier
                .fillMaxHeight()
                .horizontalScroll(horizontal)
                .width(contentWidth)
                .graphicsLayer {
                    scaleX = pinch
                    scaleY = pinch
                },
        ) {
            itemsIndexed(pages) { index, size ->
                PageItem(
                    index = index,
                    size = size,
                    width = pageWidth,
                    locations = locations.filter { it.page == index },
                    focused = focused,
                    matches = matches.filter { it.page == index },
                    current = current?.takeIf { it.page == index },
                    activeSelection = selection?.takeIf { it.page == index },
                    textLayer = textLayer,
                    isSnapshotMode = isSnapshotMode,
                    selectAfterLongPress = selectAfterLongPress,
                    onSelect = { selection = it },
                    onAddSnapshot = { rect -> onAddSnapshot(rect, index) },
                    loadPage = loadPage,
                    renderedNear = renderedNear,
                    onTapPage = onTapPage,
                )
            }
        }

        selection?.let { sel ->
            SelectionActionPalette(
                visible = true,
                onHighlight = { colorName ->
                    // The words just before the selection tell repeated quotes apart.
                    scope.launch {
                        val prefix = textLayer.range(sel.page, max(0, sel.start - PREFIX_GLYPHS), sel.start)?.text.orEmpty()
                        onHighlight(sel.text, prefix, colorName)
                    }
                    selection = null
                },
                onAddNote = {
                    onAddNote(sel.text)
                    selection = null
                },
                onCopy = {
                    clipboard.setText(AnnotatedString(sel.text))
                    selection = null
                },
                onCopyCitation = {
                    clipboard.setText(AnnotatedString("“${sel.text}” (p. ${sel.page + 1})"))
                    selection = null
                },
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = Dimens.SpaceLg + contentPadding.calculateBottomPadding()),
            )
        }
    }
}

private val MaxPageWidth = 900.dp

/** Glyphs before a selection kept as its highlight's prefix. */
private const val PREFIX_GLYPHS = 40

@Composable
private fun PageItem(
    index: Int,
    size: PageSize,
    width: Dp,
    locations: List<QuoteLocation>,
    focused: String?,
    matches: List<PdfMatch>,
    current: PdfMatch?,
    activeSelection: ActivePdfSelection?,
    textLayer: PdfTextLayer,
    isSnapshotMode: Boolean = false,
    selectAfterLongPress: Boolean = false,
    onSelect: (ActivePdfSelection?) -> Unit,
    onAddSnapshot: (PageRect) -> Unit = {},
    loadPage: suspend (Int, Int) -> ImageBitmap?,
    renderedNear: (Int, Int) -> ImageBitmap?,
    onTapPage: () -> Unit = {},
) {
    val widthPx = with(LocalDensity.current) { width.toPx() }.roundToInt()
    // A page rendered at another width (before a zoom or resize) stands in until the sharp one is
    // ready, instead of a blank page.
    val image: ImageBitmap? by produceState(renderedNear(index, widthPx), index, widthPx) {
        loadPage(index, widthPx)?.let { value = it }
    }
    val colors = QuireTheme.colors
    val scope = rememberCoroutineScope()
    var snapshotStart by remember { mutableStateOf<Offset?>(null) }
    var snapshotCurrent by remember { mutableStateOf<Offset?>(null) }

    Box(
        modifier = Modifier
            .width(width)
            .aspectRatio(size.width / size.height)
            .shadow(6.dp, RoundedCornerShape(Dimens.RadiusSmall))
            .clip(RoundedCornerShape(Dimens.RadiusSmall))
            .background(Color.White)
            .pointerInput(index, textLayer, isSnapshotMode) {
                fun points(offset: Offset): Offset = offset / (this.size.width.toFloat() / size.width)
                if (isSnapshotMode) {
                    detectDragGestures(
                        onDragStart = { offset ->
                            val at = points(offset)
                            snapshotStart = at
                            snapshotCurrent = at
                        },
                        onDrag = { change, _ ->
                            change.consume()
                            snapshotCurrent = points(change.position)
                        },
                        onDragEnd = {
                            val s = snapshotStart
                            val c = snapshotCurrent
                            if (s != null && c != null) {
                                val rect = PageRect(
                                    left = minOf(s.x, c.x),
                                    top = minOf(s.y, c.y),
                                    right = maxOf(s.x, c.x),
                                    bottom = maxOf(s.y, c.y),
                                )
                                if ((rect.right - rect.left) > 5f && (rect.bottom - rect.top) > 5f) {
                                    onAddSnapshot(rect)
                                }
                            }
                            snapshotStart = null
                            snapshotCurrent = null
                        },
                        onDragCancel = {
                            snapshotStart = null
                            snapshotCurrent = null
                        },
                    )
                } else {
                    detectTapGestures(
                        onTap = {
                            onSelect(null)
                            onTapPage()
                        },
                        onDoubleTap = { offset ->
                            val at = points(offset)
                            scope.launch {
                                val word = textLayer.wordAt(index, at.x, at.y) ?: return@launch
                                val first = textLayer.glyphAt(index, word.rect.left + 0.5f, (word.rect.top + word.rect.bottom) / 2) ?: return@launch
                                val last = textLayer.glyphAt(index, word.rect.right - 0.5f, (word.rect.top + word.rect.bottom) / 2) ?: return@launch
                                select(textLayer, index, first, last)?.let(onSelect)
                            }
                        },
                    )
                }
            }
            .then(
                if (isSnapshotMode) Modifier
                else Modifier.pointerInput(index, textLayer, selectAfterLongPress) {
                    fun points(offset: Offset): Offset = offset / (this.size.width.toFloat() / size.width)
                    var anchor: Int? = null
                    var pending: Job? = null
                    val start = { offset: Offset ->
                        val at = points(offset)
                        anchor = null
                        scope.launch { anchor = textLayer.glyphAt(index, at.x, at.y) }
                        Unit
                    }
                    val drag = { change: PointerInputChange ->
                        change.consume()
                        val at = points(change.position)
                        pending?.cancel()
                        pending = scope.launch {
                            val from = anchor ?: return@launch
                            val to = textLayer.glyphAt(index, at.x, at.y) ?: return@launch
                            select(textLayer, index, from, to)?.let(onSelect)
                        }
                    }
                    if (selectAfterLongPress) {
                        detectDragGesturesAfterLongPress(onDragStart = start, onDrag = { change, _ -> drag(change) })
                    } else {
                        detectDragGestures(onDragStart = start, onDrag = { change, _ -> drag(change) })
                    }
                }
            ),
    ) {
        image?.let {
            Image(bitmap = it, contentDescription = null, contentScale = ContentScale.FillBounds, modifier = Modifier.fillMaxSize())
        }

        Canvas(Modifier.fillMaxSize()) {
            val scale = this.size.width / size.width
            for (location in locations) {
                val isFocused = location.highlightId == focused
                location.rects.forEach { box(it, scale, colors.highlightYellow.copy(alpha = if (isFocused) 0.55f else 0.35f), if (isFocused) colors.accent else null) }
            }
            for (match in matches) {
                val isCurrent = match == current
                match.rects.forEach { box(it, scale, colors.findMatch.copy(alpha = if (isCurrent) 0.5f else 0.25f), if (isCurrent) colors.findMatch else null) }
            }
            activeSelection?.rects?.forEach { box(it, scale, SelectionFill, SelectionStroke) }

            // Snapshot Marquee Overlay
            if (snapshotStart != null && snapshotCurrent != null) {
                val s = snapshotStart!!
                val c = snapshotCurrent!!
                val left = minOf(s.x, c.x) * scale
                val top = minOf(s.y, c.y) * scale
                val w = kotlin.math.abs(c.x - s.x) * scale
                val h = kotlin.math.abs(c.y - s.y) * scale
                drawRect(color = colors.accent.copy(alpha = 0.2f), topLeft = Offset(left, top), size = Size(w, h))
                drawRect(color = colors.accent, topLeft = Offset(left, top), size = Size(w, h), style = Stroke(width = 2.dp.toPx()))
            }
        }
    }
}

/** The selection between glyphs `a` and `b` (either order, both included); `null` if no text. */
private suspend fun select(layer: PdfTextLayer, page: Int, a: Int, b: Int): ActivePdfSelection? {
    val (start, end) = minOf(a, b) to maxOf(a, b) + 1
    val range = layer.range(page, start, end) ?: return null
    if (range.text.isBlank()) return null
    return ActivePdfSelection(page, start, end, range.rects, range.text)
}

private fun DrawScope.box(r: PageRect, scale: Float, fill: Color, stroke: Color?) {
    val topLeft = Offset(r.left * scale, r.top * scale)
    val size = Size((r.right - r.left) * scale, (r.bottom - r.top) * scale)
    val corner = CornerRadius(2.dp.toPx())
    drawRoundRect(color = fill, topLeft = topLeft, size = size, cornerRadius = corner)
    if (stroke != null) drawRoundRect(color = stroke, topLeft = topLeft, size = size, cornerRadius = corner, style = Stroke(width = 1.5.dp.toPx()))
}

private val SelectionFill = Color(0x553B82F6)
private val SelectionStroke = Color(0xFF2563EB)

/**
 * Two fingers pinching: [onPinch] with each step's zoom factor, [onEnd] when they lift. One finger
 * is left alone (the list scrolls, taps and selections work as before).
 */
private fun Modifier.pinchToZoom(onPinch: (Float) -> Unit, onEnd: () -> Unit) = pointerInput(Unit) {
    awaitEachGesture {
        awaitFirstDown(requireUnconsumed = false)
        var pinched = false
        do {
            val event = awaitPointerEvent()
            if (event.changes.count { it.pressed } >= 2) {
                val factor = event.calculateZoom()
                if (factor != 1f) {
                    pinched = true
                    onPinch(factor)
                    event.changes.forEach { it.consume() }
                }
            }
        } while (event.changes.any { it.pressed })
        if (pinched) onEnd()
    }
}
