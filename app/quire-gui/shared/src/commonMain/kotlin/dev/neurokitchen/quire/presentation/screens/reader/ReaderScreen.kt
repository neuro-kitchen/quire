package dev.neurokitchen.quire.presentation.screens.reader

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import org.jetbrains.compose.resources.painterResource
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DraggableSplitter
import dev.neurokitchen.quire.presentation.screens.reader.components.AnnotationsSidebar
import dev.neurokitchen.quire.presentation.screens.reader.components.CitationHoverCard
import dev.neurokitchen.quire.presentation.screens.reader.components.FigureDrawer
import dev.neurokitchen.quire.presentation.screens.reader.components.NotesPanel
import dev.neurokitchen.quire.presentation.screens.reader.components.PdfPagesView
import dev.neurokitchen.quire.presentation.screens.reader.components.ReaderSearchBar
import dev.neurokitchen.quire.presentation.screens.reader.components.ReaderToolbar
import dev.neurokitchen.quire.presentation.screens.reader.components.TextReaderView
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.reader_loading
import dev.neurokitchen.quire.resources.reader_no_pdf
import dev.neurokitchen.quire.resources.reader_no_text
import dev.neurokitchen.quire.resources.reader_source
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel
import org.koin.core.parameter.parametersOf

/** One paper in a reader tab; `id` is its reference ID. */
@Composable
fun ReaderRoute(
    id: String,
    title: String,
    viewModel: ReaderViewModel = koinViewModel(key = "reader-$id") { parametersOf(id, title) },
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    ReaderScreen(state, viewModel::onEvent, loadPage = viewModel::page, loadFigure = viewModel::figure, textLayer = viewModel)
}

/**
 * Zotero's reader layout: annotations on the left, the PDF pages or the text in the middle, notes
 * on the right. Supports resizable panels, context-aware zoom/font scaling (Ctrl +/-), and in-reader search.
 */
@Composable
fun ReaderScreen(
    state: ReaderState,
    onEvent: (ReaderEvent) -> Unit,
    loadPage: suspend (Int, Int) -> PageImage?,
    loadFigure: suspend (String) -> String?,
    modifier: Modifier = Modifier,
    textLayer: PdfTextLayer = PdfTextLayer.None,
) {
    val colors = QuireTheme.colors
    var selection by remember(state.referenceId) { mutableStateOf<Pair<String, String>?>(null) }
    var sidebarWidthDp by remember { mutableFloatStateOf(260f) }
    var notesWidthDp by remember { mutableFloatStateOf(280f) }
    var figureDrawerWidthDp by remember { mutableFloatStateOf(320f) }
    val rootFocusRequester = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        rootFocusRequester.requestFocus()
    }

    val located = state.pdf.locations.map { it.highlightId }.toSet()

    Box(
        modifier = modifier
            .fillMaxSize()
            .focusRequester(rootFocusRequester)
            .focusable()
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown) {
                    if (event.isCtrlPressed) {
                        when (event.key) {
                            Key.Plus, Key.Equals, Key.NumPadAdd -> {
                                if (state.mode == ReaderMode.Pdf) onEvent(ReaderEvent.OnZoomIn)
                                else onEvent(ReaderEvent.OnChangeFontSize(1))
                                true
                            }
                            Key.Minus, Key.NumPadSubtract -> {
                                if (state.mode == ReaderMode.Pdf) onEvent(ReaderEvent.OnZoomOut)
                                else onEvent(ReaderEvent.OnChangeFontSize(-1))
                                true
                            }
                            Key.Zero, Key.NumPad0 -> {
                                if (state.mode == ReaderMode.Pdf) onEvent(ReaderEvent.OnZoomReset)
                                else onEvent(ReaderEvent.OnChangeFontSize(16 - state.fontSizeSp))
                                true
                            }
                            Key.F -> {
                                onEvent(ReaderEvent.OnFindToggle)
                                true
                            }
                            Key.B -> {
                                onEvent(ReaderEvent.OnToggleAnnotations)
                                true
                            }
                            else -> false
                        }
                    } else if (event.key == Key.Escape) {
                        if (state.find.isOpen) {
                            onEvent(ReaderEvent.OnFindToggle)
                            true
                        } else false
                    } else false
                } else false
            },
    ) {
        Row(Modifier.fillMaxSize()) {
            if (!state.isAnnotationsCollapsed) {
                AnnotationsSidebar(
                    highlights = state.highlights,
                    focused = state.focusedHighlight,
                    missing = if (state.mode == ReaderMode.Pdf) state.highlights.map { it.id }.filterNot { it in located }.toSet() else emptySet(),
                    onClick = { onEvent(ReaderEvent.OnHighlightClick(it)) },
                    onComment = { id, text -> onEvent(ReaderEvent.OnHighlightComment(id, text)) },
                    onRemove = { onEvent(ReaderEvent.OnHighlightRemove(it)) },
                    onCollapse = { onEvent(ReaderEvent.OnToggleAnnotations) },
                    modifier = Modifier.width(sidebarWidthDp.dp),
                )
                DraggableSplitter(
                    onDelta = { delta ->
                        sidebarWidthDp = (sidebarWidthDp + delta).coerceIn(180f, 450f)
                    },
                )
            }
            Column(Modifier.weight(1f).fillMaxHeight()) {
                ReaderToolbar(
                    mode = state.mode,
                    hasPdf = state.pdf.path != null,
                    canHighlight = selection != null,
                    reading = state.reading,
                    isAnnotationsCollapsed = state.isAnnotationsCollapsed,
                    isNotesCollapsed = state.isNotesCollapsed,
                    isFigureDrawerOpen = state.isFigureDrawerOpen,
                    isSnapshotMode = state.isSnapshotMode,
                    isSearchOpen = state.find.isOpen,
                    readerTheme = state.readerTheme,
                    readerFont = state.readerFont,
                    fontSizeSp = state.fontSizeSp,
                    onModeChange = { onEvent(ReaderEvent.OnModeChange(it)) },
                    onZoomIn = {
                        if (state.mode == ReaderMode.Pdf) onEvent(ReaderEvent.OnZoomIn)
                        else onEvent(ReaderEvent.OnChangeFontSize(1))
                    },
                    onZoomOut = {
                        if (state.mode == ReaderMode.Pdf) onEvent(ReaderEvent.OnZoomOut)
                        else onEvent(ReaderEvent.OnChangeFontSize(-1))
                    },
                    onZoomReset = {
                        if (state.mode == ReaderMode.Pdf) onEvent(ReaderEvent.OnZoomReset)
                        else onEvent(ReaderEvent.OnChangeFontSize(16 - state.fontSizeSp))
                    },
                    onHighlight = {
                        selection?.let { (quote, prefix) -> onEvent(ReaderEvent.OnHighlight(quote, prefix)) }
                        selection = null
                    },
                    onReadingChange = { onEvent(ReaderEvent.OnReadingChange(it)) },
                    onToggleAnnotations = { onEvent(ReaderEvent.OnToggleAnnotations) },
                    onToggleNotes = { onEvent(ReaderEvent.OnToggleNotes) },
                    onToggleFigureDrawer = { onEvent(ReaderEvent.OnToggleFigureDrawer) },
                    onToggleSnapshotMode = { onEvent(ReaderEvent.OnToggleSnapshotMode) },
                    onToggleSearch = { onEvent(ReaderEvent.OnFindToggle) },
                    onSetReaderTheme = { onEvent(ReaderEvent.OnSetReaderTheme(it)) },
                    onSetReaderFont = { onEvent(ReaderEvent.OnSetReaderFont(it)) },
                    onChangeFontSize = { onEvent(ReaderEvent.OnChangeFontSize(it)) },
                )
                HorizontalDivider(color = colors.paneDivider)

                // Snapshot Mode Active Guidance Banner
                if (state.isSnapshotMode) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .fillMaxWidth()
                            .background(colors.accent)
                            .padding(horizontal = Dimens.SpaceMd, vertical = 6.dp),
                    ) {
                        Icon(
                            painterResource(Res.drawable.ic_auto_fix_high),
                            contentDescription = null,
                            tint = colors.onAccent,
                            modifier = Modifier.size(16.dp),
                        )
                        Spacer(Modifier.width(8.dp))
                        Text(
                            "Snapshot Active: Drag a rectangle over any figure or paragraph on the PDF to copy it into notes",
                            style = MaterialTheme.typography.bodySmall.copy(fontWeight = FontWeight.Medium),
                            color = colors.onAccent,
                        )
                        Spacer(Modifier.weight(1f))
                        Text(
                            "Done",
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                            color = colors.onAccent,
                            modifier = Modifier
                                .clip(RoundedCornerShape(4.dp))
                                .background(colors.onAccent.copy(alpha = 0.2f))
                                .clickable { onEvent(ReaderEvent.OnToggleSnapshotMode) }
                                .padding(horizontal = 8.dp, vertical = 2.dp),
                        )
                    }
                }

                // In-Reader Search Bar
                if (state.find.isOpen) {
                    val count = state.find.count(state.mode)
                    ReaderSearchBar(
                        query = state.find.query,
                        onQueryChange = { onEvent(ReaderEvent.OnFindChange(it)) },
                        matchCount = count,
                        currentMatch = if (count == 0) 0 else state.find.current + 1,
                        onNextMatch = { onEvent(ReaderEvent.OnFindNext) },
                        onPrevMatch = { onEvent(ReaderEvent.OnFindPrevious) },
                        onClose = { onEvent(ReaderEvent.OnFindToggle) },
                    )
                }

                StatusLine(state)
                when {
                    state.mode == ReaderMode.Split && state.pdf.pages.isNotEmpty() -> {
                        var splitWeight by remember { mutableFloatStateOf(0.5f) }
                        Row(Modifier.fillMaxSize()) {
                            Box(Modifier.weight(splitWeight).fillMaxHeight()) {
                                TextReaderView(
                                    tabs = state.tabs,
                                    selectedTab = state.selectedTab,
                                    highlights = state.highlights,
                                    focused = state.focusedHighlight,
                                    readerTheme = state.readerTheme,
                                    readerFont = state.readerFont,
                                    fontSizeSp = state.fontSizeSp,
                                    onTabClick = { onEvent(ReaderEvent.OnTabClick(it)) },
                                    onSelection = { selection = it },
                                    onHighlight = { quote, prefix, color ->
                                        onEvent(ReaderEvent.OnHighlight(quote, prefix, color))
                                    },
                                    onAddNote = { body ->
                                        onEvent(ReaderEvent.OnNoteDraftChange(body))
                                        onEvent(ReaderEvent.OnNoteAdd)
                                    },
                                    loadFigure = loadFigure,
                                    findMatches = if (state.find.isOpen) state.find.textMatches else emptyList(),
                                    currentFind = state.find.current,
                                )
                            }
                            DraggableSplitter(
                                onDelta = { delta ->
                                    splitWeight = (splitWeight + delta / 800f).coerceIn(0.25f, 0.75f)
                                },
                            )
                            Box(Modifier.weight(1f - splitWeight).fillMaxHeight()) {
                                PdfPagesView(
                                    pages = state.pdf.pages,
                                    zoom = state.pdf.zoom,
                                    locations = state.pdf.locations,
                                    focused = state.focusedHighlight,
                                    loadPage = loadPage,
                                    readerTheme = state.readerTheme,
                                    textLayer = textLayer,
                                    matches = if (state.find.isOpen) state.find.pdfMatches else emptyList(),
                                    currentMatch = state.find.current,
                                    isSnapshotMode = state.isSnapshotMode,
                                    onHighlight = { quote, prefix, color ->
                                        onEvent(ReaderEvent.OnHighlight(quote, prefix, color))
                                    },
                                    onAddNote = { body ->
                                        onEvent(ReaderEvent.OnNoteDraftChange(body))
                                        onEvent(ReaderEvent.OnNoteAdd)
                                    },
                                    onAddSnapshot = { rect, page ->
                                        onEvent(ReaderEvent.OnNoteDraftChange("[Area Snapshot p. ${page + 1}: (${rect.left.toInt()}, ${rect.top.toInt()}) - (${rect.right.toInt()}, ${rect.bottom.toInt()})]"))
                                        onEvent(ReaderEvent.OnNoteAdd)
                                    },
                                )
                            }
                        }
                    }
                    state.mode == ReaderMode.Pdf && state.pdf.pages.isNotEmpty() -> PdfPagesView(
                        pages = state.pdf.pages,
                        zoom = state.pdf.zoom,
                        locations = state.pdf.locations,
                        focused = state.focusedHighlight,
                        loadPage = loadPage,
                        readerTheme = state.readerTheme,
                        textLayer = textLayer,
                        matches = if (state.find.isOpen) state.find.pdfMatches else emptyList(),
                        currentMatch = state.find.current,
                        isSnapshotMode = state.isSnapshotMode,
                        onHighlight = { quote, prefix, color ->
                            onEvent(ReaderEvent.OnHighlight(quote, prefix, color))
                        },
                        onAddNote = { body ->
                            onEvent(ReaderEvent.OnNoteDraftChange(body))
                            onEvent(ReaderEvent.OnNoteAdd)
                        },
                        onAddSnapshot = { rect, page ->
                            onEvent(ReaderEvent.OnNoteDraftChange("[Area Snapshot p. ${page + 1}: (${rect.left.toInt()}, ${rect.top.toInt()}) - (${rect.right.toInt()}, ${rect.bottom.toInt()})]"))
                            onEvent(ReaderEvent.OnNoteAdd)
                        },
                    )
                    state.isLoading && state.tabs.isEmpty() -> Message(stringResource(Res.string.reader_loading))
                    else -> TextReaderView(
                        tabs = state.tabs,
                        selectedTab = state.selectedTab,
                        highlights = state.highlights,
                        focused = state.focusedHighlight,
                        readerTheme = state.readerTheme,
                        readerFont = state.readerFont,
                        fontSizeSp = state.fontSizeSp,
                        onTabClick = { onEvent(ReaderEvent.OnTabClick(it)) },
                        onSelection = { selection = it },
                        onHighlight = { quote, prefix, color ->
                            onEvent(ReaderEvent.OnHighlight(quote, prefix, color))
                        },
                        onAddNote = { body ->
                            onEvent(ReaderEvent.OnNoteDraftChange(body))
                            onEvent(ReaderEvent.OnNoteAdd)
                        },
                        loadFigure = loadFigure,
                        findMatches = if (state.find.isOpen) state.find.textMatches else emptyList(),
                        currentFind = state.find.current,
                    )
                }
            }
            if (!state.isNotesCollapsed) {
                DraggableSplitter(
                    onDelta = { delta ->
                        notesWidthDp = (notesWidthDp - delta).coerceIn(200f, 500f)
                    },
                )
                NotesPanel(
                    notes = state.notes,
                    draft = state.noteDraft,
                    onDraftChange = { onEvent(ReaderEvent.OnNoteDraftChange(it)) },
                    onAdd = { onEvent(ReaderEvent.OnNoteAdd) },
                    onRemove = { onEvent(ReaderEvent.OnNoteRemove(it)) },
                    onCollapse = { onEvent(ReaderEvent.OnToggleNotes) },
                    modifier = Modifier.width(notesWidthDp.dp),
                )
            }
            if (state.isFigureDrawerOpen) {
                DraggableSplitter(
                    onDelta = { delta ->
                        figureDrawerWidthDp = (figureDrawerWidthDp - delta).coerceIn(240f, 550f)
                    },
                )
                FigureDrawer(
                    isOpen = true,
                    tabs = state.tabs,
                    loadFigure = loadFigure,
                    onClose = { onEvent(ReaderEvent.OnToggleFigureDrawer) },
                    modifier = Modifier.width(figureDrawerWidthDp.dp),
                )
            }
        }

        // Citation Hover / Popover Card
        CitationHoverCard(
            citation = state.activeCitation,
            onDismissRequest = { onEvent(ReaderEvent.OnShowCitation(null)) },
        )
    }
}

private val SidebarWidth = 260.dp
private val NotesWidth = 280.dp

/** Where the text came from, why there is none, or what failed. */
@Composable
private fun StatusLine(state: ReaderState) {
    val colors = QuireTheme.colors
    val (text, isError) = when {
        state.error != null -> state.error to true
        state.mode == ReaderMode.Pdf && state.pdf.error != null -> state.pdf.error to true
        state.mode == ReaderMode.Text && state.textError != null -> stringResource(Res.string.reader_no_text, state.textError) to false
        state.mode == ReaderMode.Text && state.source != null -> stringResource(Res.string.reader_source, state.source) to false
        state.mode == ReaderMode.Pdf && state.pdf.path == null && !state.isLoading -> stringResource(Res.string.reader_no_pdf) to false
        else -> return
    }
    Text(
        text,
        style = MaterialTheme.typography.bodySmall,
        color = if (isError) colors.accentRed else colors.fillSecondary,
        modifier = Modifier.padding(horizontal = Dimens.SpaceMd, vertical = 4.dp),
    )
}

@Composable
private fun Message(text: String) {
    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Text(text, style = MaterialTheme.typography.bodyLarge, color = QuireTheme.colors.fillSecondary)
    }
}
