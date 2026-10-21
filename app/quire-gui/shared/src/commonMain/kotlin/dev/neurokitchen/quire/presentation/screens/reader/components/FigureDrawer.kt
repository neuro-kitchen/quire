package dev.neurokitchen.quire.presentation.screens.reader.components

import dev.neurokitchen.quire.resources.reader_rows
import dev.neurokitchen.quire.resources.reader_open_beside
import androidx.compose.material3.TextButton
import dev.neurokitchen.quire.presentation.screens.reader.Pinned
import org.jetbrains.compose.resources.stringResource
import dev.neurokitchen.quire.resources.figures_finding
import kotlinx.coroutines.withContext
import kotlinx.coroutines.Dispatchers
import dev.neurokitchen.quire.domain.model.PdfFigure
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import dev.neurokitchen.quire.domain.model.BlockKind
import dev.neurokitchen.quire.domain.model.PaperTab
import dev.neurokitchen.quire.domain.model.PaperTabKind
import dev.neurokitchen.quire.domain.model.TextBlock
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mapping.loadImageFile
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_inventory_2
import org.jetbrains.compose.resources.painterResource

data class ExtractedFigure(
    val id: String,
    val title: String,
    val caption: String,
    val imageUrl: String,
    /** Found on the PDF's page from its caption (no image in the text): cropped from the page. */
    val pdf: PdfFigure? = null,
)

data class ExtractedTable(
    val id: String,
    val title: String,
    val rows: List<List<String>>,
)

enum class FigureDockSide { Left, Right }

@Composable
fun FigureDrawer(
    isOpen: Boolean,
    tabs: List<PaperTab>,
    loadFigure: suspend (String) -> String?,
    onClose: () -> Unit,
    dockSide: FigureDockSide = FigureDockSide.Right,
    onToggleDockSide: () -> Unit = {},
    modifier: Modifier = Modifier,
    /** Figures found on the PDF's pages; `null` while they are looked for (no PDF: empty). */
    pdfFigures: List<PdfFigure>? = emptyList(),
    loadPdfFigure: suspend (PdfFigure) -> ImageBitmap? = { null },
    /** Opens a figure or table in its own pane beside the text or the PDF. */
    onPin: (Pinned) -> Unit = {},
) {
    if (!isOpen) return

    val colors = QuireTheme.colors
    var selectedTab by remember { mutableStateOf(0) } // 0: Figures, 1: Tables
    var enlarged by remember { mutableStateOf<ExtractedFigure?>(null) }
    val loadImage: suspend (ExtractedFigure) -> ImageBitmap? = { figure -> figureImage(figure, loadFigure, loadPdfFigure) }

    // Extract figures and tables from paper tabs
    val textFigures = remember(tabs) {
        val list = mutableListOf<ExtractedFigure>()
        var figIndex = 1
        tabs.forEach { tab ->
            tab.blocks.forEachIndexed { i, block ->
                if (block.kind is BlockKind.Image) {
                    val caption = tab.blocks.getOrNull(i + 1)?.text ?: ""
                    list.add(
                        ExtractedFigure(
                            id = "fig-$figIndex",
                            title = "Figure $figIndex",
                            caption = caption,
                            imageUrl = block.kind.url,
                        )
                    )
                    figIndex++
                } else if (tab.kind == PaperTabKind.Figures && block.kind is BlockKind.Heading) {
                    val caption = tab.blocks.getOrNull(i + 1)?.text ?: ""
                    list.add(
                        ExtractedFigure(
                            id = "fig-$figIndex",
                            title = block.text.ifEmpty { "Figure $figIndex" },
                            caption = caption,
                            imageUrl = "",
                        )
                    )
                    figIndex++
                }
            }
        }
        list
    }

    // A text read from the PDF has captions but no images: the figures found on the pages instead.
    val shownFigures = remember(textFigures, pdfFigures) {
        if (textFigures.any { it.imageUrl.isNotEmpty() } || pdfFigures.isNullOrEmpty()) textFigures
        else pdfFigures.mapIndexed { i, f -> ExtractedFigure("pdf-$i", f.label, f.caption, imageUrl = "", pdf = f) }
    }

    val tables = remember(tabs) {
        val list = mutableListOf<ExtractedTable>()
        var tblIndex = 1
        tabs.forEach { tab ->
            var currentHeading = ""
            val currentRows = mutableListOf<List<String>>()

            fun flushCurrentTable() {
                if (currentRows.isNotEmpty()) {
                    val title = currentHeading.ifEmpty { "Table $tblIndex" }
                    list.add(
                        ExtractedTable(
                            id = "tbl-$tblIndex",
                            title = title,
                            rows = currentRows.toList(),
                        )
                    )
                    tblIndex++
                    currentRows.clear()
                    currentHeading = ""
                }
            }

            tab.blocks.forEach { block ->
                when (block.kind) {
                    is BlockKind.Heading -> {
                        flushCurrentTable()
                        if (block.text.contains("table", ignoreCase = true) || tab.kind == PaperTabKind.Tables) {
                            currentHeading = block.text
                        }
                    }
                    is BlockKind.TableRow -> {
                        // One word per cell, as the engine keeps them (joined text loses the columns).
                        val cells = block.words.map { it.text }
                        if (cells.any { it.isNotBlank() }) {
                            currentRows.add(cells)
                        }
                    }
                    else -> {
                        if (block.text.contains("table", ignoreCase = true)) {
                            flushCurrentTable()
                            currentHeading = block.text
                        } else if (currentRows.size >= 15 && block.text.isBlank()) {
                            flushCurrentTable()
                        }
                    }
                }
            }
            flushCurrentTable()
        }
        list
    }

    val borderShape = if (dockSide == FigureDockSide.Right) {
        RoundedCornerShape(topStart = Dimens.RadiusBase, bottomStart = Dimens.RadiusBase)
    } else {
        RoundedCornerShape(topEnd = Dimens.RadiusBase, bottomEnd = Dimens.RadiusBase)
    }

    Column(
        modifier = modifier
            .fillMaxHeight()
            .background(colors.sidepane)
            .border(1.dp, colors.border, borderShape),
    ) {
        // Drawer Header
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 14.dp, vertical = 10.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = "FIGURES & TABLES",
                    style = MaterialTheme.typography.labelSmall.copy(
                        fontWeight = FontWeight.Bold,
                        letterSpacing = 1.sp,
                        fontSize = 11.sp,
                    ),
                    color = colors.fillPrimary,
                )
            }

            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                // Dock Toggle button (VS Code style movable dock)
                Box(
                    contentAlignment = Alignment.Center,
                    modifier = Modifier
                        .height(24.dp)
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .background(colors.fillQuarternary.copy(alpha = 0.6f))
                        .clickable(onClick = onToggleDockSide)
                        .padding(horizontal = 7.dp),
                ) {
                    Text(
                        text = if (dockSide == FigureDockSide.Right) "← Dock Left" else "Dock Right →",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.Medium),
                        color = colors.fillSecondary,
                    )
                }

                Box(
                    contentAlignment = Alignment.Center,
                    modifier = Modifier
                        .size(24.dp)
                        .clip(CircleShape)
                        .background(colors.fillQuarternary.copy(alpha = 0.6f))
                        .clickable(onClick = onClose),
                ) {
                    Icon(
                        painterResource(Res.drawable.ic_close),
                        contentDescription = "Close Drawer",
                        tint = colors.fillSecondary,
                        modifier = Modifier.size(14.dp),
                    )
                }
            }
        }

        // Subtabs: Figures / Tables
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 14.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            DrawerTabChip("Figures (${shownFigures.size})", selected = selectedTab == 0) { selectedTab = 0 }
            DrawerTabChip("Tables (${tables.size})", selected = selectedTab == 1) { selectedTab = 1 }
        }

        HorizontalDivider(color = colors.border.copy(alpha = 0.4f), modifier = Modifier.padding(top = 8.dp))

        // Content List
        if (selectedTab == 0) {
            if (shownFigures.isEmpty() && pdfFigures == null) {
                DrawerEmptyState(title = "", message = stringResource(Res.string.figures_finding))
            } else if (shownFigures.isEmpty()) {
                DrawerEmptyState(
                    title = "No Extracted Figures",
                    message = "This document does not contain standalone raster figures in its text layer.\n\nTip: Use the Snapshot tool in the toolbar to capture figures directly from the PDF into notes.",
                )
            } else {
                LazyColumn(
                    contentPadding = PaddingValues(12.dp),
                    verticalArrangement = Arrangement.spacedBy(14.dp),
                    modifier = Modifier.fillMaxSize(),
                ) {
                    itemsIndexed(shownFigures) { _, fig ->
                        FigureCard(
                            figure = fig,
                            loadImage = loadImage,
                            onEnlarge = { enlarged = fig },
                            onPin = { onPin(Pinned.Figure(fig)) },
                        )
                    }
                }
            }
        } else {
            if (tables.isEmpty()) {
                DrawerEmptyState(
                    title = "No Extracted Tables",
                    message = "No structured tables were extracted from this document's text layer.",
                )
            } else {
                LazyColumn(
                    contentPadding = PaddingValues(12.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                    modifier = Modifier.fillMaxSize(),
                ) {
                    itemsIndexed(tables) { _, tbl ->
                        TableCard(table = tbl, onPin = { onPin(Pinned.Table(tbl)) })
                    }
                }
            }
        }
    }

    // High-Res Image Lightbox Dialog
    enlarged?.let { figure ->
        Dialog(onDismissRequest = { enlarged = null }) {
            Surface(
                shape = RoundedCornerShape(12.dp),
                color = colors.background,
                shadowElevation = 24.dp,
                modifier = Modifier
                    .padding(16.dp)
                    .clip(RoundedCornerShape(12.dp)),
            ) {
                Column(Modifier.padding(16.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.End,
                    ) {
                        Box(
                            contentAlignment = Alignment.Center,
                            modifier = Modifier
                                .size(28.dp)
                                .clip(CircleShape)
                                .background(colors.fillQuarternary)
                                .clickable { enlarged = null },
                        ) {
                            Icon(
                                painterResource(Res.drawable.ic_close),
                                contentDescription = "Close Lightbox",
                                tint = colors.fillPrimary,
                                modifier = Modifier.size(16.dp),
                            )
                        }
                    }
                    val img: ImageBitmap? by produceState<ImageBitmap?>(null, figure) { value = loadImage(figure) }
                    if (img != null) {
                        Image(
                            bitmap = img!!,
                            contentDescription = "Enlarged Figure",
                            modifier = Modifier
                                .fillMaxWidth()
                                .heightIn(max = 600.dp),
                        )
                    } else {
                        Box(
                            contentAlignment = Alignment.Center,
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(200.dp),
                        ) {
                            Text("Loading figure...", color = colors.fillTertiary)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun FigureCard(
    figure: ExtractedFigure,
    loadImage: suspend (ExtractedFigure) -> ImageBitmap?,
    onEnlarge: () -> Unit,
    onPin: () -> Unit,
) {
    val colors = QuireTheme.colors
    val image: ImageBitmap? by produceState<ImageBitmap?>(null, figure) { value = loadImage(figure) }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(Dimens.RadiusBase))
            .background(colors.background)
            .border(1.dp, colors.border.copy(alpha = 0.5f), RoundedCornerShape(Dimens.RadiusBase))
            .clickable(onClick = onEnlarge)
            .padding(10.dp),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                text = figure.title,
                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold, fontSize = 11.sp),
                color = colors.accent,
            )
            OpenBeside(onPin)
        }

        Spacer(Modifier.height(6.dp))

        if (image != null) {
            Image(
                bitmap = image!!,
                contentDescription = figure.title,
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(max = 180.dp)
                    .clip(RoundedCornerShape(Dimens.RadiusSmall)),
            )
        } else {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(90.dp)
                    .background(colors.fillQuarternary.copy(alpha = 0.4f)),
            ) {
                Text(
                    "Figure Preview",
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.fillTertiary,
                )
            }
        }

        if (figure.caption.isNotBlank()) {
            Spacer(Modifier.height(6.dp))
            Text(
                text = figure.caption,
                style = MaterialTheme.typography.bodySmall.copy(fontSize = 11.sp, lineHeight = 15.sp),
                color = colors.fillSecondary,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun TableCard(table: ExtractedTable, onPin: () -> Unit) {
    val colors = QuireTheme.colors
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(Dimens.RadiusBase))
            .background(colors.background)
            .border(1.dp, colors.border.copy(alpha = 0.5f), RoundedCornerShape(Dimens.RadiusBase))
            .padding(10.dp),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(
                text = table.title,
                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold, fontSize = 11.sp),
                color = colors.accent,
                modifier = Modifier.weight(1f, fill = false),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.width(8.dp))
            Text(
                text = stringResource(Res.string.reader_rows, table.rows.size),
                style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp),
                color = colors.fillTertiary,
            )
            Spacer(Modifier.weight(1f))
            OpenBeside(onPin)
        }
        Spacer(Modifier.height(8.dp))
        PaperTable(
            rows = table.rows,
            textStyle = MaterialTheme.typography.bodySmall,
            textColor = colors.fillPrimary,
            lineColor = colors.border.copy(alpha = 0.5f),
            stripeColor = colors.sidepane.copy(alpha = 0.6f),
            maxColumnWidth = 220.dp,
        )
    }
}

@Composable
private fun DrawerTabChip(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
) {
    val colors = QuireTheme.colors
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(Dimens.RadiusFull))
            .background(if (selected) colors.accent else colors.fillQuarternary.copy(alpha = 0.5f))
            .clickable(onClick = onClick)
            .padding(horizontal = 10.dp, vertical = 3.dp),
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall.copy(
                fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                fontSize = 11.sp,
            ),
            color = if (selected) colors.onAccent else colors.fillSecondary,
        )
    }
}

@Composable
private fun DrawerEmptyState(title: String, message: String) {
    val colors = QuireTheme.colors
    Box(
        contentAlignment = Alignment.Center,
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Icon(
                painterResource(Res.drawable.ic_inventory_2),
                contentDescription = null,
                tint = colors.fillTertiary,
                modifier = Modifier.size(32.dp),
            )
            Text(
                text = title,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.SemiBold,
                color = colors.fillPrimary,
            )
            Text(
                text = message,
                style = MaterialTheme.typography.bodySmall,
                color = colors.fillSecondary,
                textAlign = TextAlign.Center,
            )
        }
    }
}

/**
 * A figure's image: cut from its PDF page, or the text's image file decoded (off the UI thread;
 * a large figure decoded in composition stalled the reader). `null` when it cannot be had.
 */
internal suspend fun figureImage(
    figure: ExtractedFigure,
    loadFigure: suspend (String) -> String?,
    loadPdfFigure: suspend (PdfFigure) -> ImageBitmap?,
): ImageBitmap? = when {
    figure.pdf != null -> loadPdfFigure(figure.pdf)
    figure.imageUrl.isNotEmpty() -> loadFigure(figure.imageUrl)?.let { path -> withContext(Dispatchers.Default) { loadImageFile(path) } }
    else -> null
}

/** "Open Beside": the figure or table in its own pane beside the text or the PDF. */
@Composable
private fun OpenBeside(onClick: () -> Unit) {
    TextButton(onClick = onClick, contentPadding = PaddingValues(horizontal = Dimens.SpaceXs), modifier = Modifier.height(24.dp)) {
        Text(stringResource(Res.string.reader_open_beside), style = MaterialTheme.typography.labelSmall)
    }
}
