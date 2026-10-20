package dev.neurokitchen.quire.presentation.screens.reader.components

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
import org.jetbrains.compose.resources.painterResource

data class ExtractedFigure(
    val id: String,
    val title: String,
    val caption: String,
    val imageUrl: String,
)

data class ExtractedTable(
    val id: String,
    val title: String,
    val content: String,
)

@Composable
fun FigureDrawer(
    isOpen: Boolean,
    tabs: List<PaperTab>,
    loadFigure: suspend (String) -> String?,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    if (!isOpen) return

    val colors = QuireTheme.colors
    var selectedTab by remember { mutableStateOf(0) } // 0: Figures, 1: Tables
    var enlargedImageUrl by remember { mutableStateOf<String?>(null) }

    // Extract figures and tables from paper tabs
    val figures = remember(tabs) {
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
                }
            }
        }
        list
    }

    val tables = remember(tabs) {
        val list = mutableListOf<ExtractedTable>()
        var tblIndex = 1
        tabs.forEach { tab ->
            tab.blocks.forEach { block ->
                if (block.kind is BlockKind.TableRow) {
                    list.add(
                        ExtractedTable(
                            id = "tbl-$tblIndex",
                            title = "Table $tblIndex",
                            content = block.text,
                        )
                    )
                    tblIndex++
                }
            }
        }
        list
    }

    Column(
        modifier = modifier
            .fillMaxHeight()
            .background(colors.sidepane)
            .border(1.dp, colors.border, RoundedCornerShape(topStart = Dimens.RadiusBase, bottomStart = Dimens.RadiusBase)),
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

        // Subtabs: Figures / Tables
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 14.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            DrawerTabChip("Figures (${figures.size})", selected = selectedTab == 0) { selectedTab = 0 }
            DrawerTabChip("Tables (${tables.size})", selected = selectedTab == 1) { selectedTab = 1 }
        }

        HorizontalDivider(color = colors.border.copy(alpha = 0.4f), modifier = Modifier.padding(top = 8.dp))

        // Content List
        if (selectedTab == 0) {
            if (figures.isEmpty()) {
                DrawerEmptyState("No extracted figures in this document")
            } else {
                LazyColumn(
                    contentPadding = PaddingValues(12.dp),
                    verticalArrangement = Arrangement.spacedBy(14.dp),
                    modifier = Modifier.fillMaxSize(),
                ) {
                    itemsIndexed(figures) { _, fig ->
                        FigureCard(
                            figure = fig,
                            loadFigure = loadFigure,
                            onEnlarge = { enlargedImageUrl = fig.imageUrl },
                        )
                    }
                }
            }
        } else {
            if (tables.isEmpty()) {
                DrawerEmptyState("No extracted tables in this document")
            } else {
                LazyColumn(
                    contentPadding = PaddingValues(12.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                    modifier = Modifier.fillMaxSize(),
                ) {
                    itemsIndexed(tables) { _, tbl ->
                        TableCard(table = tbl)
                    }
                }
            }
        }
    }

    // High-Res Image Lightbox Dialog
    enlargedImageUrl?.let { url ->
        Dialog(onDismissRequest = { enlargedImageUrl = null }) {
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
                                .clickable { enlargedImageUrl = null },
                        ) {
                            Icon(
                                painterResource(Res.drawable.ic_close),
                                contentDescription = "Close Lightbox",
                                tint = colors.fillPrimary,
                                modifier = Modifier.size(16.dp),
                            )
                        }
                    }
                    val img: ImageBitmap? by produceState<ImageBitmap?>(null, url) {
                        value = loadFigure(url)?.let(::loadImageFile)
                    }
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
    loadFigure: suspend (String) -> String?,
    onEnlarge: () -> Unit,
) {
    val colors = QuireTheme.colors
    val image: ImageBitmap? by produceState<ImageBitmap?>(null, figure.imageUrl) {
        value = loadFigure(figure.imageUrl)?.let(::loadImageFile)
    }

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
            Text(
                text = "Click to enlarge",
                style = MaterialTheme.typography.labelSmall.copy(fontSize = 9.sp),
                color = colors.fillTertiary,
            )
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
private fun TableCard(table: ExtractedTable) {
    val colors = QuireTheme.colors
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(Dimens.RadiusBase))
            .background(colors.background)
            .border(1.dp, colors.border.copy(alpha = 0.5f), RoundedCornerShape(Dimens.RadiusBase))
            .padding(10.dp),
    ) {
        Text(
            text = table.title,
            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold, fontSize = 11.sp),
            color = colors.accent,
        )
        Spacer(Modifier.height(6.dp))
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .background(colors.fillQuarternary.copy(alpha = 0.3f))
                .padding(8.dp),
        ) {
            Text(
                text = table.content,
                style = MaterialTheme.typography.bodySmall.copy(
                    fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace,
                    fontSize = 10.sp,
                ),
                color = colors.fillPrimary,
                maxLines = 4,
                overflow = TextOverflow.Ellipsis,
            )
        }
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
private fun DrawerEmptyState(message: String) {
    val colors = QuireTheme.colors
    Box(
        contentAlignment = Alignment.Center,
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
    ) {
        Text(
            text = message,
            style = MaterialTheme.typography.bodySmall,
            color = colors.fillTertiary,
        )
    }
}
