package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import dev.chrisbanes.haze.HazeInput
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.blur.hazeBlur
import dev.chrisbanes.haze.blur.materials.HazeMaterials
import dev.chrisbanes.haze.hazeSource
import dev.chrisbanes.haze.rememberHazeState
import dev.neurokitchen.quire.domain.model.PdfFigure
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import dev.neurokitchen.quire.presentation.designsystem.readerColors
import dev.neurokitchen.quire.presentation.screens.reader.PdfTextLayer
import dev.neurokitchen.quire.presentation.screens.reader.ReaderEvent
import dev.neurokitchen.quire.presentation.screens.reader.ReaderMode
import dev.neurokitchen.quire.presentation.screens.reader.ReaderState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.common_back
import dev.neurokitchen.quire.resources.find_close_search_esc
import dev.neurokitchen.quire.resources.find_find_in_document
import dev.neurokitchen.quire.resources.ic_arrow_back
import dev.neurokitchen.quire.resources.ic_chevron_right
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_inventory_2
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.ic_sticky_note_2
import dev.neurokitchen.quire.resources.reader_display
import dev.neurokitchen.quire.resources.reader_display_font
import dev.neurokitchen.quire.resources.reader_display_page
import dev.neurokitchen.quire.resources.reader_display_pdf
import dev.neurokitchen.quire.resources.reader_display_size
import dev.neurokitchen.quire.resources.reader_display_text
import dev.neurokitchen.quire.resources.reader_display_theme
import dev.neurokitchen.quire.resources.reader_display_view
import dev.neurokitchen.quire.resources.reader_display_zoom
import dev.neurokitchen.quire.resources.reader_display_zoom_fit
import dev.neurokitchen.quire.resources.reader_figures
import dev.neurokitchen.quire.resources.reader_find_count
import dev.neurokitchen.quire.resources.reader_find_next
import dev.neurokitchen.quire.resources.reader_find_previous
import dev.neurokitchen.quire.resources.reader_highlights
import dev.neurokitchen.quire.resources.reader_loading
import dev.neurokitchen.quire.resources.reader_notes
import dev.neurokitchen.quire.resources.reader_page_n_of
import dev.neurokitchen.quire.resources.reader_percent_read
import dev.neurokitchen.quire.resources.reader_view_figure
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import kotlin.math.roundToInt

/**
 * The reader on a phone. The page or the text fills the screen; a tap shows or hides frosted bars
 * over it (Haze): on top, back, the title and find; at the bottom, a page slider and a floating
 * toolbar with Highlights, Figures, Notes and Aa (Material 3 Expressive: a toolbar, no navigation
 * bar, on a screen opened from a list). Panels open as bottom sheets; a figure opens full screen.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MobileReaderView(
    state: ReaderState,
    onEvent: (ReaderEvent) -> Unit,
    loadPage: suspend (Int, Int) -> ImageBitmap?,
    loadFigure: suspend (String) -> String?,
    renderedNear: (Int, Int) -> ImageBitmap?,
    textLayer: PdfTextLayer,
    onSelectionChange: (Pair<String, String>?) -> Unit,
    onBack: (() -> Unit)?,
    modifier: Modifier = Modifier,
    loadPdfFigure: suspend (PdfFigure) -> ImageBitmap? = { null },
) {
    val haze = rememberHazeState()
    var isChromeVisible by remember { mutableStateOf(true) }
    var isDisplayOpen by remember { mutableStateOf(false) }
    var position by remember(state.referenceId, state.mode) { mutableIntStateOf(1) }
    var total by remember(state.referenceId, state.mode) { mutableIntStateOf(1) }
    var jump by remember(state.referenceId, state.mode) { mutableStateOf<Int?>(null) }
    val isPdf = state.mode == ReaderMode.Pdf && state.pdf.pages.isNotEmpty()

    // The bars' room: the content scrolls under them and stays readable at both ends.
    val status = WindowInsets.statusBars.asPaddingValues().calculateTopPadding()
    val navigation = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()
    val padding = PaddingValues(top = status + ReaderTopBarHeight, bottom = navigation + ReaderBottomHeight)
    val toggleChrome = { isChromeVisible = !isChromeVisible }
    val progress = { current: Int, count: Int ->
        position = current
        total = count.coerceAtLeast(1)
    }

    Box(modifier.fillMaxSize().background(readerColors(state.readerTheme).background)) {
        Box(Modifier.fillMaxSize().hazeSource(haze)) {
            when {
                isPdf -> PdfPagesView(
                    pages = state.pdf.pages,
                    zoom = state.pdf.zoom,
                    locations = state.pdf.locations,
                    focused = state.focusedHighlight,
                    loadPage = loadPage,
                    renderedNear = renderedNear,
                    readerTheme = state.readerTheme,
                    textLayer = textLayer,
                    matches = if (state.find.isOpen) state.find.pdfMatches else emptyList(),
                    currentMatch = state.find.current,
                    isSnapshotMode = state.isSnapshotMode,
                    onHighlight = { quote, prefix, color -> onEvent(ReaderEvent.OnHighlight(quote, prefix, color)) },
                    onAddNote = { body ->
                        onEvent(ReaderEvent.OnNoteDraftChange(body))
                        onEvent(ReaderEvent.OnNoteAdd)
                    },
                    onProgressChange = progress,
                    onTapPage = toggleChrome,
                    contentPadding = padding,
                    jumpToPage = jump,
                    onPinchZoom = { onEvent(ReaderEvent.OnSetZoom(it)) },
                    selectAfterLongPress = true,
                )
                state.isLoading && state.tabs.isEmpty() -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text(stringResource(Res.string.reader_loading), color = QuireTheme.colors.fillSecondary, style = MaterialTheme.typography.bodyLarge)
                }
                else -> TextReaderView(
                    tabs = state.tabs,
                    selectedTab = state.selectedTab,
                    highlights = state.highlights,
                    focused = state.focusedHighlight,
                    readerTheme = state.readerTheme,
                    readerFont = state.readerFont,
                    fontSizeSp = state.fontSizeSp,
                    onTabClick = { onEvent(ReaderEvent.OnTabClick(it)) },
                    onSelection = onSelectionChange,
                    onHighlight = { quote, prefix, color -> onEvent(ReaderEvent.OnHighlight(quote, prefix, color)) },
                    onAddNote = { body ->
                        onEvent(ReaderEvent.OnNoteDraftChange(body))
                        onEvent(ReaderEvent.OnNoteAdd)
                    },
                    loadFigure = loadFigure,
                    findMatches = if (state.find.isOpen) state.find.textMatches else emptyList(),
                    currentFind = state.find.current,
                    onProgressChange = progress,
                    onTapPage = toggleChrome,
                    contentPadding = padding,
                    jumpToBlock = jump,
                )
            }
        }

        AnimatedVisibility(
            visible = isChromeVisible || state.find.isOpen,
            enter = fadeIn(tween(180)) + slideInVertically(tween(180)) { -it },
            exit = fadeOut(tween(180)) + slideOutVertically(tween(180)) { -it },
            modifier = Modifier.align(Alignment.TopCenter),
        ) {
            if (state.find.isOpen) {
                FindBar(state, onEvent, haze)
            } else {
                TopBar(state.title, onBack, onFind = { onEvent(ReaderEvent.OnFindToggle) }, haze)
            }
        }

        AnimatedVisibility(
            visible = isChromeVisible,
            enter = fadeIn(tween(180)) + slideInVertically(tween(180)) { it },
            exit = fadeOut(tween(180)) + slideOutVertically(tween(180)) { it },
            modifier = Modifier.align(Alignment.BottomCenter),
        ) {
            BottomControls(
                isPdf = isPdf,
                position = position,
                total = if (isPdf) state.pdf.pages.size else total,
                highlights = state.highlights.size,
                notes = state.notes.size,
                haze = haze,
                onJump = { jump = it },
                onHighlights = { onEvent(ReaderEvent.OnToggleAnnotations) },
                onFigures = { onEvent(ReaderEvent.OnToggleFigureDrawer) },
                onNotes = { onEvent(ReaderEvent.OnToggleNotes) },
                onDisplay = { isDisplayOpen = true },
            )
        }

        if (isDisplayOpen) DisplaySheet(state, isPdf, onEvent, onDismiss = { isDisplayOpen = false })
        if (!state.isAnnotationsCollapsed) {
            PanelSheet(onDismiss = { onEvent(ReaderEvent.OnToggleAnnotations) }) {
                val located = state.pdf.locations.map { it.highlightId }.toSet()
                AnnotationsSidebar(
                    highlights = state.highlights,
                    focused = state.focusedHighlight,
                    missing = if (isPdf) state.highlights.map { it.id }.filterNot { it in located }.toSet() else emptySet(),
                    onClick = {
                        onEvent(ReaderEvent.OnHighlightClick(it))
                        onEvent(ReaderEvent.OnToggleAnnotations)
                    },
                    onComment = { id, text -> onEvent(ReaderEvent.OnHighlightComment(id, text)) },
                    onRemove = { onEvent(ReaderEvent.OnHighlightRemove(it)) },
                    onCollapse = { onEvent(ReaderEvent.OnToggleAnnotations) },
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
        if (!state.isNotesCollapsed) {
            PanelSheet(onDismiss = { onEvent(ReaderEvent.OnToggleNotes) }) {
                NotesPanel(
                    notes = state.notes,
                    draft = state.noteDraft,
                    onDraftChange = { onEvent(ReaderEvent.OnNoteDraftChange(it)) },
                    onAdd = { onEvent(ReaderEvent.OnNoteAdd) },
                    onRemove = { onEvent(ReaderEvent.OnNoteRemove(it)) },
                    onCollapse = { onEvent(ReaderEvent.OnToggleNotes) },
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
        if (state.isFigureDrawerOpen) {
            PanelSheet(onDismiss = { onEvent(ReaderEvent.OnToggleFigureDrawer) }) {
                FigureDrawer(
                    isOpen = true,
                    tabs = state.tabs,
                    loadFigure = loadFigure,
                    pdfFigures = if (state.pdf.path == null) emptyList() else state.pdf.figures,
                    loadPdfFigure = loadPdfFigure,
                    onClose = { onEvent(ReaderEvent.OnToggleFigureDrawer) },
                    onPin = {
                        onEvent(ReaderEvent.OnPin(it))
                        onEvent(ReaderEvent.OnToggleFigureDrawer)
                    },
                    pinLabel = stringResource(Res.string.reader_view_figure),
                    showDockToggle = false,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
        // "View" on a figure or table: full screen (a phone has no room for a pane beside the page).
        state.pinned?.let { pinned ->
            Dialog(
                onDismissRequest = { onEvent(ReaderEvent.OnPin(null)) },
                properties = DialogProperties(usePlatformDefaultWidth = false),
            ) {
                PinnedPane(
                    pinned = pinned,
                    loadFigure = loadFigure,
                    loadPdfFigure = loadPdfFigure,
                    onClose = { onEvent(ReaderEvent.OnPin(null)) },
                    modifier = Modifier.fillMaxSize().statusBarsPadding().navigationBarsPadding(),
                )
            }
        }
    }
}

/** The top bar's height, without the status bar. */
private val ReaderTopBarHeight = 56.dp

/** The bottom controls' height (slider and toolbar), without the system navigation bar. */
private val ReaderBottomHeight = 112.dp

@Composable
private fun Frosted(haze: HazeState, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    Box(modifier.hazeBlur(HazeInput.Sources(haze), HazeMaterials.thin(QuireTheme.colors.background))) { content() }
}

@Composable
private fun TopBar(title: String, onBack: (() -> Unit)?, onFind: () -> Unit, haze: HazeState) {
    val colors = QuireTheme.colors
    Frosted(haze, Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth().statusBarsPadding().height(ReaderTopBarHeight).padding(horizontal = Dimens.SpaceMin),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (onBack != null) {
                IconButton(onClick = onBack) {
                    Icon(painterResource(Res.drawable.ic_arrow_back), contentDescription = stringResource(Res.string.common_back), tint = colors.fillPrimary)
                }
            }
            Text(
                title,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = colors.fillPrimary,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f).padding(horizontal = Dimens.SpaceXs),
            )
            IconButton(onClick = onFind) {
                Icon(painterResource(Res.drawable.ic_search), contentDescription = stringResource(Res.string.find_find_in_document), tint = colors.fillPrimary)
            }
        }
    }
}

/** Find, in the top bar: the field, how many matches, previous and next (48 dp each), close. */
@Composable
private fun FindBar(state: ReaderState, onEvent: (ReaderEvent) -> Unit, haze: HazeState) {
    val colors = QuireTheme.colors
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { focus.requestFocus() }
    val count = state.find.count(state.mode)
    Frosted(haze, Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth().statusBarsPadding().height(ReaderTopBarHeight).padding(horizontal = Dimens.SpaceMin),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(onClick = { onEvent(ReaderEvent.OnFindToggle) }) {
                Icon(painterResource(Res.drawable.ic_close), contentDescription = stringResource(Res.string.find_close_search_esc), tint = colors.fillPrimary)
            }
            TextField(
                value = state.find.query,
                onValueChange = { onEvent(ReaderEvent.OnFindChange(it)) },
                placeholder = { Text(stringResource(Res.string.find_find_in_document)) },
                singleLine = true,
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                keyboardActions = KeyboardActions(onSearch = { onEvent(ReaderEvent.OnFindNext) }),
                colors = TextFieldDefaults.colors(
                    focusedContainerColor = Color.Transparent,
                    unfocusedContainerColor = Color.Transparent,
                    focusedIndicatorColor = Color.Transparent,
                    unfocusedIndicatorColor = Color.Transparent,
                ),
                modifier = Modifier.weight(1f).focusRequester(focus),
            )
            if (state.find.query.isNotBlank()) {
                Text(
                    stringResource(Res.string.reader_find_count, if (count == 0) 0 else state.find.current + 1, count),
                    style = MaterialTheme.typography.labelMedium,
                    color = colors.fillSecondary,
                )
            }
            IconButton(onClick = { onEvent(ReaderEvent.OnFindPrevious) }, enabled = count > 0) {
                Icon(painterResource(Res.drawable.ic_chevron_right), contentDescription = stringResource(Res.string.reader_find_previous), tint = colors.fillPrimary, modifier = Modifier.rotate(-90f))
            }
            IconButton(onClick = { onEvent(ReaderEvent.OnFindNext) }, enabled = count > 0) {
                Icon(painterResource(Res.drawable.ic_chevron_right), contentDescription = stringResource(Res.string.reader_find_next), tint = colors.fillPrimary, modifier = Modifier.rotate(90f))
            }
        }
    }
}

/**
 * Bottom controls: where you are (page n of N, or % for text) with a slider to move there, and a
 * floating toolbar (Highlights, Figures, Notes, Aa), frosted.
 */
@Composable
private fun BottomControls(
    isPdf: Boolean,
    position: Int,
    total: Int,
    highlights: Int,
    notes: Int,
    haze: HazeState,
    onJump: (Int) -> Unit,
    onHighlights: () -> Unit,
    onFigures: () -> Unit,
    onNotes: () -> Unit,
    onDisplay: () -> Unit,
) {
    val colors = QuireTheme.colors
    val last = total.coerceAtLeast(1)
    var dragging by remember { mutableStateOf(false) }
    var value by remember { mutableFloatStateOf(position.toFloat()) }
    if (!dragging) value = position.coerceIn(1, last).toFloat()
    // 0% at the start, 100% on the last page or block.
    val percent = if (last <= 1) 100 else (((value - 1f) / (last - 1f)) * 100).roundToInt()
    Column(
        Modifier.fillMaxWidth().navigationBarsPadding().padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceXs),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
    ) {
        Frosted(haze, Modifier.fillMaxWidth().clip(RoundedCornerShape(Dimens.RadiusLarge))) {
            Row(Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceMd), verticalAlignment = Alignment.CenterVertically) {
                Text(
                    if (isPdf) stringResource(Res.string.reader_page_n_of, value.roundToInt(), last) else stringResource(Res.string.reader_percent_read, percent),
                    style = MaterialTheme.typography.labelMedium,
                    color = colors.fillSecondary,
                    modifier = Modifier.widthIn(min = 92.dp),
                )
                if (last > 1) {
                    Slider(
                        value = value,
                        onValueChange = {
                            dragging = true
                            value = it
                        },
                        onValueChangeFinished = {
                            dragging = false
                            onJump(value.roundToInt() - 1)
                        },
                        valueRange = 1f..last.toFloat(),
                        colors = SliderDefaults.colors(thumbColor = colors.accent, activeTrackColor = colors.accent, inactiveTrackColor = colors.fillTertiary),
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }
        Frosted(
            haze,
            Modifier
                .clip(CircleShape)
                .border(1.dp, colors.paneDivider.copy(alpha = 0.4f), CircleShape),
        ) {
            Row(Modifier.padding(horizontal = Dimens.SpaceXs, vertical = Dimens.SpaceMin), horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceMin)) {
                ToolbarAction(Res.drawable.ic_sticky_note_2, stringResource(Res.string.reader_highlights), highlights, onHighlights)
                ToolbarAction(Res.drawable.ic_inventory_2, stringResource(Res.string.reader_figures), null, onFigures)
                ToolbarAction(Res.drawable.ic_description, stringResource(Res.string.reader_notes), notes, onNotes)
                Box(
                    Modifier.size(48.dp).clip(CircleShape).clickable(onClick = onDisplay),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("Aa", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold, color = colors.accent)
                }
            }
        }
    }
}

/** A toolbar action: icon, a count badge if any, the label for TalkBack. */
@Composable
private fun ToolbarAction(icon: org.jetbrains.compose.resources.DrawableResource, label: String, count: Int?, onClick: () -> Unit) {
    val colors = QuireTheme.colors
    Box(Modifier.size(48.dp).clip(CircleShape).clickable(onClickLabel = label, onClick = onClick), contentAlignment = Alignment.Center) {
        Icon(painterResource(icon), contentDescription = label, tint = colors.fillPrimary, modifier = Modifier.size(22.dp))
        if (count != null && count > 0) {
            Text(
                count.toString(),
                style = MaterialTheme.typography.labelSmall,
                color = colors.onAccent,
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(top = 6.dp, end = 4.dp)
                    .clip(CircleShape)
                    .background(colors.accent)
                    .padding(horizontal = 5.dp),
            )
        }
    }
}

/** A reader panel (highlights, figures, notes) as a bottom sheet, most of the screen tall. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun PanelSheet(onDismiss: () -> Unit, content: @Composable () -> Unit) {
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = QuireTheme.colors.sidepane,
    ) {
        Box(Modifier.fillMaxWidth().fillMaxHeight(0.85f)) { content() }
    }
}

/**
 * Aa: how the paper shows. PDF or text; for the PDF, zoom; for the text, size and typeface; the
 * page colours for both. (Reading status is on the paper's details.)
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun DisplaySheet(state: ReaderState, isPdf: Boolean, onEvent: (ReaderEvent) -> Unit, onDismiss: () -> Unit) {
    val colors = QuireTheme.colors
    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        containerColor = colors.cardBackground,
    ) {
        Column(
            Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceLg).padding(bottom = Dimens.SpaceLg),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
        ) {
            Text(stringResource(Res.string.reader_display), style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.SemiBold, color = colors.fillPrimary)
            if (state.pdf.path != null) {
                Label(stringResource(Res.string.reader_display_view))
                SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                    listOf(ReaderMode.Pdf to Res.string.reader_display_pdf, ReaderMode.Text to Res.string.reader_display_text).forEachIndexed { i, (mode, label) ->
                        SegmentedButton(
                            selected = (state.mode == ReaderMode.Text) == (mode == ReaderMode.Text),
                            onClick = { onEvent(ReaderEvent.OnModeChange(mode)) },
                            shape = SegmentedButtonDefaults.itemShape(i, 2),
                        ) { Text(stringResource(label)) }
                    }
                }
            }
            if (isPdf) {
                Label(stringResource(Res.string.reader_display_zoom))
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                    FilterChip(selected = false, onClick = { onEvent(ReaderEvent.OnZoomOut) }, label = { Text("−") })
                    Text("${(state.pdf.zoom * 100).roundToInt()}%", style = MaterialTheme.typography.titleMedium, color = colors.fillPrimary, modifier = Modifier.widthIn(min = 56.dp))
                    FilterChip(selected = false, onClick = { onEvent(ReaderEvent.OnZoomIn) }, label = { Text("+") })
                    Spacer(Modifier.weight(1f))
                    FilterChip(selected = state.pdf.zoom == 1f, onClick = { onEvent(ReaderEvent.OnZoomReset) }, label = { Text(stringResource(Res.string.reader_display_zoom_fit)) })
                }
            } else {
                Label(stringResource(Res.string.reader_display_size))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("A", style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
                    Slider(
                        value = state.fontSizeSp.toFloat(),
                        onValueChange = { onEvent(ReaderEvent.OnChangeFontSize(it.roundToInt() - state.fontSizeSp)) },
                        valueRange = 12f..28f,
                        steps = 7,
                        colors = SliderDefaults.colors(thumbColor = colors.accent, activeTrackColor = colors.accent, inactiveTrackColor = colors.fillTertiary),
                        modifier = Modifier.weight(1f).padding(horizontal = Dimens.SpaceSm),
                    )
                    Text("A", style = MaterialTheme.typography.titleLarge, color = colors.fillPrimary)
                }
                Label(stringResource(Res.string.reader_display_font))
                SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                    ReaderFont.entries.forEachIndexed { i, font ->
                        SegmentedButton(
                            selected = font == state.readerFont,
                            onClick = { onEvent(ReaderEvent.OnSetReaderFont(font)) },
                            shape = SegmentedButtonDefaults.itemShape(i, ReaderFont.entries.size),
                        ) {
                            Text(
                                font.title,
                                fontFamily = when (font) {
                                    ReaderFont.Serif -> FontFamily.Serif
                                    ReaderFont.Sans -> FontFamily.SansSerif
                                    ReaderFont.Mono -> FontFamily.Monospace
                                },
                            )
                        }
                    }
                }
            }
            Label(stringResource(if (isPdf) Res.string.reader_display_page else Res.string.reader_display_theme))
            Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm)) {
                ReaderTheme.entries.forEach { theme ->
                    val themeColors = readerColors(theme)
                    val selected = theme == state.readerTheme
                    Box(
                        Modifier
                            .size(48.dp)
                            .clip(CircleShape)
                            .background(themeColors.background)
                            .border(if (selected) 3.dp else 1.dp, if (selected) colors.accent else colors.paneDivider, CircleShape)
                            .clickable(onClickLabel = theme.title) { onEvent(ReaderEvent.OnSetReaderTheme(theme)) },
                        contentAlignment = Alignment.Center,
                    ) {
                        Text("Aa", color = themeColors.text, style = MaterialTheme.typography.labelLarge, fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal)
                    }
                }
            }
            Text(
                ReaderTheme.entries.first { it == state.readerTheme }.title,
                style = MaterialTheme.typography.bodySmall,
                color = colors.fillSecondary,
            )
        }
    }
}

@Composable
private fun Label(text: String) {
    Text(text, style = MaterialTheme.typography.labelLarge, color = QuireTheme.colors.fillSecondary, modifier = Modifier.padding(top = Dimens.SpaceXs))
}
