package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import dev.neurokitchen.quire.presentation.screens.reader.PdfTextLayer
import dev.neurokitchen.quire.presentation.screens.reader.ReaderEvent
import dev.neurokitchen.quire.presentation.screens.reader.ReaderMode
import dev.neurokitchen.quire.presentation.screens.reader.ReaderState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_arrow_back
import dev.neurokitchen.quire.resources.ic_draft
import dev.neurokitchen.quire.resources.ic_inventory_2
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.ic_sticky_note_2
import dev.neurokitchen.quire.resources.reader_loading
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import kotlin.math.roundToInt

/**
 * Mobile Reader View inspired by Kindle and Google Play Books:
 * - Immersive full-screen reading canvas by default
 * - Single touch / tap on canvas toggles controls and smoothly minimizes the page (scale down)
 * - Minimal top header: Back arrow, document title, Kindle "Aa" display options, search
 * - Minimal bottom scrubber: page progress, scrubber bar, quick drawer buttons
 * - Modal bottom sheet for "Aa" text & layout configuration (fonts, sizes, themes, mode)
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
    selection: Pair<String, String>?,
    onSelectionChange: (Pair<String, String>?) -> Unit,
    readingProgressCurrent: Int,
    onProgressCurrentChange: (Int) -> Unit,
    readingProgressTotal: Int,
    onProgressTotalChange: (Int) -> Unit,
    onBack: (() -> Unit)?,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    var isChromeVisible by remember { mutableStateOf(false) }
    var isAaSheetOpen by remember { mutableStateOf(false) }

    val safeTotal = (if (state.mode == ReaderMode.Pdf) state.pdf.pages.size else readingProgressTotal).coerceAtLeast(1)
    val safeCurrent = readingProgressCurrent.coerceIn(1, safeTotal)
    val progressFraction = (safeCurrent.toFloat() / safeTotal.toFloat()).coerceIn(0f, 1f)
    val progressPercent = (progressFraction * 100).toInt()

    // Smooth page scale down & rounded corners when controls are shown (Kindle page overview effect)
    val pageScale by animateFloatAsState(
        targetValue = if (isChromeVisible) 0.91f else 1.0f,
        animationSpec = tween(durationMillis = 280, easing = FastOutSlowInEasing),
    )
    val cornerRadius by animateDpAsState(
        targetValue = if (isChromeVisible) 20.dp else 0.dp,
        animationSpec = tween(durationMillis = 280, easing = FastOutSlowInEasing),
    )
    val pageElevation by animateDpAsState(
        targetValue = if (isChromeVisible) 24.dp else 0.dp,
        animationSpec = tween(durationMillis = 280, easing = FastOutSlowInEasing),
    )

    Box(
        modifier = modifier
            .fillMaxSize()
            .background(colors.background),
    ) {
        // Document Reading Canvas Container
        Box(
            modifier = Modifier
                .fillMaxSize()
                .pointerInput(Unit) {
                    detectTapGestures(
                        onTap = {
                            isChromeVisible = !isChromeVisible
                        },
                    )
                },
            contentAlignment = Alignment.Center,
        ) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .graphicsLayer {
                        scaleX = pageScale
                        scaleY = pageScale
                        shadowElevation = pageElevation.toPx()
                        shape = RoundedCornerShape(cornerRadius)
                        clip = true
                    },
            ) {
                when {
                    state.mode == ReaderMode.Pdf && state.pdf.pages.isNotEmpty() -> {
                        PdfPagesView(
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
                            onProgressChange = { cur, tot ->
                                onProgressCurrentChange(cur)
                                onProgressTotalChange(tot)
                            },
                            onTapPage = { isChromeVisible = !isChromeVisible },
                        )
                    }
                    state.isLoading && state.tabs.isEmpty() -> {
                        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            Text(
                                stringResource(Res.string.reader_loading),
                                color = colors.fillSecondary,
                                style = MaterialTheme.typography.bodyLarge,
                            )
                        }
                    }
                    else -> {
                        TextReaderView(
                            tabs = state.tabs,
                            selectedTab = state.selectedTab,
                            highlights = state.highlights,
                            focused = state.focusedHighlight,
                            readerTheme = state.readerTheme,
                            readerFont = state.readerFont,
                            fontSizeSp = state.fontSizeSp,
                            onTabClick = { onEvent(ReaderEvent.OnTabClick(it)) },
                            onSelection = onSelectionChange,
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
                            onProgressChange = { cur, tot ->
                                onProgressCurrentChange(cur)
                                onProgressTotalChange(tot)
                            },
                            onTapPage = { isChromeVisible = !isChromeVisible },
                        )
                    }
                }
            }
        }

        // Top Minimal Chrome Header (Kindle Style)
        AnimatedVisibility(
            visible = isChromeVisible,
            enter = fadeIn(tween(200)) + slideInVertically(tween(200)) { -it },
            exit = fadeOut(tween(200)) + slideOutVertically(tween(200)) { -it },
            modifier = Modifier.align(Alignment.TopCenter),
        ) {
            MobileReaderTopBar(
                title = state.title,
                onBack = onBack,
                onAaClick = { isAaSheetOpen = true },
                onSearchClick = { onEvent(ReaderEvent.OnFindToggle) },
                isSearchOpen = state.find.isOpen,
                modifier = Modifier.fillMaxWidth(),
            )
        }

        // In-Reader Search Bar (if triggered)
        if (isChromeVisible && state.find.isOpen) {
            val count = state.find.count(state.mode)
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .align(Alignment.TopCenter)
                    .statusBarsPadding()
                    .padding(top = 64.dp)
                    .padding(horizontal = 16.dp),
            ) {
                ReaderSearchBar(
                    query = state.find.query,
                    onQueryChange = { onEvent(ReaderEvent.OnFindChange(it)) },
                    currentMatch = state.find.current ?: 0,
                    matchCount = count,
                    onNextMatch = { onEvent(ReaderEvent.OnFindNext) },
                    onPrevMatch = { onEvent(ReaderEvent.OnFindPrevious) },
                    onClose = { onEvent(ReaderEvent.OnFindToggle) },
                )
            }
        }

        // Bottom Minimal Scrubber Bar (Kindle Style)
        AnimatedVisibility(
            visible = isChromeVisible,
            enter = fadeIn(tween(200)) + slideInVertically(tween(200)) { it },
            exit = fadeOut(tween(200)) + slideOutVertically(tween(200)) { it },
            modifier = Modifier.align(Alignment.BottomCenter),
        ) {
            MobileReaderBottomScrubber(
                current = safeCurrent,
                total = safeTotal,
                progressFraction = progressFraction,
                progressPercent = progressPercent,
                highlightsCount = state.highlights.size,
                notesCount = state.notes.size,
                onToggleHighlights = { onEvent(ReaderEvent.OnToggleAnnotations) },
                onToggleNotes = { onEvent(ReaderEvent.OnToggleNotes) },
                onToggleFigures = { onEvent(ReaderEvent.OnToggleFigureDrawer) },
                onAaClick = { isAaSheetOpen = true },
                modifier = Modifier.fillMaxWidth(),
            )
        }

        // Kindle "Aa" Text & Layout Options Modal Bottom Sheet
        if (isAaSheetOpen) {
            KindleDisplayOptionsSheet(
                mode = state.mode,
                onModeChange = { onEvent(ReaderEvent.OnModeChange(it)) },
                hasPdf = state.pdf.path != null,
                fontSizeSp = state.fontSizeSp,
                onChangeFontSize = { onEvent(ReaderEvent.OnChangeFontSize(it)) },
                readerFont = state.readerFont,
                onSetReaderFont = { onEvent(ReaderEvent.OnSetReaderFont(it)) },
                readerTheme = state.readerTheme,
                onSetReaderTheme = { onEvent(ReaderEvent.OnSetReaderTheme(it)) },
                reading = state.reading,
                onReadingChange = { onEvent(ReaderEvent.OnReadingChange(it)) },
                onDismiss = { isAaSheetOpen = false },
            )
        }
    }
}

/** Minimal, elegant top bar shown when controls are active */
@Composable
private fun MobileReaderTopBar(
    title: String,
    onBack: (() -> Unit)?,
    onAaClick: () -> Unit,
    onSearchClick: () -> Unit,
    isSearchOpen: Boolean,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors

    Box(
        modifier = modifier
            .fillMaxWidth()
            .background(colors.cardBackground.copy(alpha = 0.96f))
            .statusBarsPadding(),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .fillMaxWidth()
                .height(56.dp)
                .padding(horizontal = 14.dp),
        ) {
            // Circular Back Button
            if (onBack != null) {
                Box(
                    modifier = Modifier
                        .size(40.dp)
                        .clip(CircleShape)
                        .background(colors.background)
                        .clickable(onClick = onBack),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_arrow_back),
                        contentDescription = "Back",
                        tint = colors.fillPrimary,
                        modifier = Modifier.size(20.dp),
                    )
                }
            }

            Spacer(Modifier.width(12.dp))

            // Title
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium.copy(
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 15.sp,
                ),
                color = colors.fillPrimary,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )

            Spacer(Modifier.width(8.dp))

            // Kindle "Aa" Text Options Button
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(CircleShape)
                    .background(colors.background)
                    .clickable(onClick = onAaClick),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "Aa",
                    style = MaterialTheme.typography.titleMedium.copy(
                        fontWeight = FontWeight.ExtraBold,
                        fontSize = 16.sp,
                    ),
                    color = colors.accent,
                )
            }

            Spacer(Modifier.width(8.dp))

            // Search Icon
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(CircleShape)
                    .background(if (isSearchOpen) colors.accent.copy(alpha = 0.16f) else colors.background)
                    .clickable(onClick = onSearchClick),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    painter = painterResource(Res.drawable.ic_search),
                    contentDescription = "Search in Document",
                    tint = if (isSearchOpen) colors.accent else colors.fillPrimary,
                    modifier = Modifier.size(19.dp),
                )
            }
        }
    }
}

/** Minimal Kindle-style bottom progress scrubber and tool icons */
@Composable
private fun MobileReaderBottomScrubber(
    current: Int,
    total: Int,
    progressFraction: Float,
    progressPercent: Int,
    highlightsCount: Int,
    notesCount: Int,
    onToggleHighlights: () -> Unit,
    onToggleNotes: () -> Unit,
    onToggleFigures: () -> Unit,
    onAaClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors

    Box(
        modifier = modifier
            .fillMaxWidth()
            .background(colors.cardBackground.copy(alpha = 0.96f))
            .navigationBarsPadding(),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 10.dp),
        ) {
            // Reading Progress Indicator & Fraction
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth().padding(bottom = 6.dp),
            ) {
                Text(
                    text = "Page $current of $total",
                    style = MaterialTheme.typography.labelSmall.copy(
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Medium,
                    ),
                    color = colors.fillSecondary,
                )
                Text(
                    text = "$progressPercent% read",
                    style = MaterialTheme.typography.labelSmall.copy(
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                    ),
                    color = colors.accent,
                )
            }

            // Thin Kindle Scrubber Bar
            LinearProgressIndicator(
                progress = { progressFraction },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(4.dp)
                    .clip(RoundedCornerShape(Dimens.RadiusFull)),
                color = colors.accent,
                trackColor = colors.background,
            )

            Spacer(Modifier.height(12.dp))

            // Clean, Uncrowded Tool Actions
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceAround,
                modifier = Modifier.fillMaxWidth(),
            ) {
                // Annotations / Highlights
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier
                        .clip(RoundedCornerShape(8.dp))
                        .clickable(onClick = onToggleHighlights)
                        .padding(horizontal = 12.dp, vertical = 4.dp),
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_sticky_note_2),
                        contentDescription = "Highlights",
                        tint = colors.fillPrimary,
                        modifier = Modifier.size(20.dp),
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        text = if (highlightsCount > 0) "Highlights ($highlightsCount)" else "Highlights",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
                        color = colors.fillSecondary,
                    )
                }

                // Figures
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier
                        .clip(RoundedCornerShape(8.dp))
                        .clickable(onClick = onToggleFigures)
                        .padding(horizontal = 12.dp, vertical = 4.dp),
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_inventory_2),
                        contentDescription = "Figures",
                        tint = colors.fillPrimary,
                        modifier = Modifier.size(20.dp),
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        text = "Figures",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
                        color = colors.fillSecondary,
                    )
                }

                // Notes
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier
                        .clip(RoundedCornerShape(8.dp))
                        .clickable(onClick = onToggleNotes)
                        .padding(horizontal = 12.dp, vertical = 4.dp),
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_draft),
                        contentDescription = "Notes",
                        tint = colors.fillPrimary,
                        modifier = Modifier.size(20.dp),
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        text = if (notesCount > 0) "Notes ($notesCount)" else "Notes",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
                        color = colors.fillSecondary,
                    )
                }

                // Display Options
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    modifier = Modifier
                        .clip(RoundedCornerShape(8.dp))
                        .clickable(onClick = onAaClick)
                        .padding(horizontal = 12.dp, vertical = 4.dp),
                ) {
                    Text(
                        text = "Aa",
                        style = MaterialTheme.typography.titleMedium.copy(
                            fontWeight = FontWeight.Bold,
                            fontSize = 17.sp,
                        ),
                        color = colors.accent,
                    )
                    Spacer(Modifier.height(1.dp))
                    Text(
                        text = "Display",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
                        color = colors.fillSecondary,
                    )
                }
            }
        }
    }
}

/**
 * Kindle-style "Aa" Display Options Modal Bottom Sheet
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun KindleDisplayOptionsSheet(
    mode: ReaderMode,
    onModeChange: (ReaderMode) -> Unit,
    hasPdf: Boolean,
    fontSizeSp: Int,
    onChangeFontSize: (Int) -> Unit,
    readerFont: ReaderFont,
    onSetReaderFont: (ReaderFont) -> Unit,
    readerTheme: ReaderTheme,
    onSetReaderTheme: (ReaderTheme) -> Unit,
    reading: ReadingStatus,
    onReadingChange: (ReadingStatus) -> Unit,
    onDismiss: () -> Unit,
) {
    val colors = QuireTheme.colors
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = colors.cardBackground,
        dragHandle = {
            Box(
                modifier = Modifier
                    .padding(vertical = 10.dp)
                    .width(36.dp)
                    .height(4.dp)
                    .clip(RoundedCornerShape(2.dp))
                    .background(colors.fillTertiary),
            )
        },
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp)
                .padding(bottom = 32.dp),
        ) {
            Text(
                text = "Display & Text Options",
                style = MaterialTheme.typography.titleMedium.copy(
                    fontWeight = FontWeight.Bold,
                    fontSize = 18.sp,
                ),
                color = colors.fillPrimary,
            )

            Spacer(Modifier.height(16.dp))

            // Section 1: View Mode (PDF vs Clean Text)
            if (hasPdf) {
                Text(
                    text = "VIEW MODE",
                    style = MaterialTheme.typography.labelSmall.copy(
                        fontWeight = FontWeight.Bold,
                        letterSpacing = 1.sp,
                        fontSize = 11.sp,
                    ),
                    color = colors.fillSecondary,
                )
                Spacer(Modifier.height(8.dp))
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(Dimens.RadiusMedium))
                        .background(colors.background)
                        .padding(4.dp),
                ) {
                    val isPdf = mode == ReaderMode.Pdf
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .clip(RoundedCornerShape(Dimens.RadiusSmall))
                            .background(if (isPdf) colors.accent else Color.Transparent)
                            .clickable { onModeChange(ReaderMode.Pdf) }
                            .padding(vertical = 8.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = "PDF Document",
                            style = MaterialTheme.typography.labelMedium.copy(
                                fontWeight = if (isPdf) FontWeight.Bold else FontWeight.Medium,
                            ),
                            color = if (isPdf) colors.onAccent else colors.fillSecondary,
                        )
                    }

                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .clip(RoundedCornerShape(Dimens.RadiusSmall))
                            .background(if (!isPdf) colors.accent else Color.Transparent)
                            .clickable { onModeChange(ReaderMode.Text) }
                            .padding(vertical = 8.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = "Reader Text",
                            style = MaterialTheme.typography.labelMedium.copy(
                                fontWeight = if (!isPdf) FontWeight.Bold else FontWeight.Medium,
                            ),
                            color = if (!isPdf) colors.onAccent else colors.fillSecondary,
                        )
                    }
                }
                Spacer(Modifier.height(18.dp))
            }

            // Section 2: Font Size Slider (Kindle A- / A+)
            Text(
                text = "FONT SIZE",
                style = MaterialTheme.typography.labelSmall.copy(
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 1.sp,
                    fontSize = 11.sp,
                ),
                color = colors.fillSecondary,
            )
            Spacer(Modifier.height(8.dp))
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(Dimens.RadiusMedium))
                    .background(colors.background)
                    .padding(horizontal = 16.dp, vertical = 6.dp),
            ) {
                Text(
                    text = "A",
                    style = MaterialTheme.typography.bodySmall.copy(
                        fontWeight = FontWeight.Bold,
                        fontSize = 13.sp,
                    ),
                    color = colors.fillSecondary,
                    modifier = Modifier.clickable { onChangeFontSize(-1) },
                )
                Spacer(Modifier.width(12.dp))
                Slider(
                    value = fontSizeSp.toFloat(),
                    onValueChange = { onChangeFontSize(it.roundToInt() - fontSizeSp) },
                    valueRange = 12f..28f,
                    steps = 7,
                    colors = SliderDefaults.colors(
                        thumbColor = colors.accent,
                        activeTrackColor = colors.accent,
                        inactiveTrackColor = colors.fillTertiary,
                    ),
                    modifier = Modifier.weight(1f),
                )
                Spacer(Modifier.width(12.dp))
                Text(
                    text = "A",
                    style = MaterialTheme.typography.titleLarge.copy(
                        fontWeight = FontWeight.Bold,
                        fontSize = 22.sp,
                    ),
                    color = colors.fillPrimary,
                    modifier = Modifier.clickable { onChangeFontSize(1) },
                )
            }

            Spacer(Modifier.height(18.dp))

            // Section 3: Font Family (Serif, Sans, Mono)
            Text(
                text = "TYPEFACE",
                style = MaterialTheme.typography.labelSmall.copy(
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 1.sp,
                    fontSize = 11.sp,
                ),
                color = colors.fillSecondary,
            )
            Spacer(Modifier.height(8.dp))
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                modifier = Modifier.fillMaxWidth(),
            ) {
                ReaderFont.entries.forEach { font ->
                    val isSelected = font == readerFont
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .clip(RoundedCornerShape(Dimens.RadiusMedium))
                            .background(if (isSelected) colors.accent else colors.background)
                            .clickable { onSetReaderFont(font) }
                            .padding(vertical = 10.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = font.title,
                            style = MaterialTheme.typography.labelMedium.copy(
                                fontFamily = when (font) {
                                    ReaderFont.Serif -> FontFamily.Serif
                                    ReaderFont.Sans -> FontFamily.SansSerif
                                    ReaderFont.Mono -> FontFamily.Monospace
                                },
                                fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                            ),
                            color = if (isSelected) colors.onAccent else colors.fillPrimary,
                        )
                    }
                }
            }

            Spacer(Modifier.height(18.dp))

            // Section 4: Reader Theme
            Text(
                text = "LIGHTING & THEME",
                style = MaterialTheme.typography.labelSmall.copy(
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 1.sp,
                    fontSize = 11.sp,
                ),
                color = colors.fillSecondary,
            )
            Spacer(Modifier.height(8.dp))
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                modifier = Modifier.fillMaxWidth(),
            ) {
                listOf(
                    ReaderTheme.Dark to "Dark",
                    ReaderTheme.Sepia to "Sepia",
                    ReaderTheme.Nord to "Nord",
                    ReaderTheme.Light to "Light",
                ).forEach { (th, label) ->
                    val isSelected = th == readerTheme
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .clip(RoundedCornerShape(Dimens.RadiusMedium))
                            .background(
                                when (th) {
                                    ReaderTheme.Dark -> Color(0xFF18181B)
                                    ReaderTheme.Sepia -> Color(0xFFFBF0D9)
                                    ReaderTheme.Nord -> Color(0xFF2E3440)
                                    ReaderTheme.Light -> Color(0xFFFFFFFF)
                                    else -> colors.background
                                },
                            )
                            .clickable { onSetReaderTheme(th) }
                            .padding(vertical = 10.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = label,
                            style = MaterialTheme.typography.labelSmall.copy(
                                fontWeight = if (isSelected) FontWeight.ExtraBold else FontWeight.Medium,
                            ),
                            color = when (th) {
                                ReaderTheme.Light, ReaderTheme.Sepia -> Color(0xFF1E293B)
                                else -> Color(0xFFF4F4F5)
                            },
                        )
                    }
                }
            }

            Spacer(Modifier.height(18.dp))

            // Section 5: Reading Status
            Text(
                text = "MARK AS",
                style = MaterialTheme.typography.labelSmall.copy(
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 1.sp,
                    fontSize = 11.sp,
                ),
                color = colors.fillSecondary,
            )
            Spacer(Modifier.height(8.dp))
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                modifier = Modifier.fillMaxWidth(),
            ) {
                listOf(
                    ReadingStatus.Unread to "Unread",
                    ReadingStatus.Reading to "Reading",
                    ReadingStatus.Read to "Read",
                ).forEach { (status, label) ->
                    val isSelected = status == reading
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .clip(RoundedCornerShape(Dimens.RadiusMedium))
                            .background(if (isSelected) colors.accent else colors.background)
                            .clickable { onReadingChange(status) }
                            .padding(vertical = 10.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = label,
                            style = MaterialTheme.typography.labelSmall.copy(
                                fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                            ),
                            color = if (isSelected) colors.onAccent else colors.fillSecondary,
                        )
                    }
                }
            }
        }
    }
}
