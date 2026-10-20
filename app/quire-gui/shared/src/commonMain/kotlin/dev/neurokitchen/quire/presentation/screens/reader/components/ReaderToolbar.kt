package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.designsystem.ReaderTheme
import dev.neurokitchen.quire.presentation.designsystem.readerColors
import dev.neurokitchen.quire.presentation.screens.reader.ReaderMode
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_inventory_2
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.ic_sticky_note_2
import dev.neurokitchen.quire.resources.reader_highlight
import dev.neurokitchen.quire.resources.reader_mode_pdf
import dev.neurokitchen.quire.resources.reader_mode_text
import dev.neurokitchen.quire.resources.reader_zoom_in
import dev.neurokitchen.quire.resources.reader_zoom_out
import dev.neurokitchen.quire.resources.reader_zoom_reset
import dev.neurokitchen.quire.resources.reading_read
import dev.neurokitchen.quire.resources.reading_reading
import dev.neurokitchen.quire.resources.reading_unread
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * Quire Reader Toolbar:
 * - Panel toggles on the left: Annotations toggle, Notes toggle
 * - Mode switch (PDF / Text)
 * - Zoom controls for PDF / Highlight for text
 * - Typography & Theme menu ("Aa") with 6 themes and 3 font families
 * - In-reader search trigger (Ctrl+F)
 * - Reading status pills on the far right
 */
@Composable
fun ReaderToolbar(
    mode: ReaderMode,
    hasPdf: Boolean,
    canHighlight: Boolean,
    reading: ReadingStatus,
    isAnnotationsCollapsed: Boolean = false,
    isNotesCollapsed: Boolean = false,
    isSearchOpen: Boolean = false,
    readerTheme: ReaderTheme = ReaderTheme.Dark,
    readerFont: ReaderFont = ReaderFont.Serif,
    fontSizeSp: Int = 16,
    onModeChange: (ReaderMode) -> Unit,
    onZoomIn: () -> Unit,
    onZoomOut: () -> Unit,
    onZoomReset: () -> Unit,
    onHighlight: () -> Unit,
    onReadingChange: (ReadingStatus) -> Unit,
    onToggleAnnotations: () -> Unit = {},
    onToggleNotes: () -> Unit = {},
    onToggleSearch: () -> Unit = {},
    onSetReaderTheme: (ReaderTheme) -> Unit = {},
    onSetReaderFont: (ReaderFont) -> Unit = {},
    onChangeFontSize: (Int) -> Unit = {},
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    var typographyMenuOpen by remember { mutableStateOf(false) }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        modifier = modifier
            .fillMaxWidth()
            .height(Dimens.ToolbarHeight)
            .background(colors.toolbar)
            .padding(horizontal = Dimens.SpaceSm),
    ) {
        // Toggle Annotations Sidebar (Left side)
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier
                .size(Dimens.ToolbarButton)
                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                .background(if (!isAnnotationsCollapsed) colors.accent.copy(alpha = 0.15f) else Color.Transparent)
                .clickable(onClick = onToggleAnnotations),
        ) {
            Icon(
                painterResource(Res.drawable.ic_inventory_2),
                contentDescription = "Toggle Annotations Panel",
                tint = if (!isAnnotationsCollapsed) colors.accent else colors.fillSecondary,
                modifier = Modifier.size(16.dp),
            )
        }

        // Toggle Notes Panel (Moved to the LEFT side per user request)
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier
                .size(Dimens.ToolbarButton)
                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                .background(if (!isNotesCollapsed) colors.accent.copy(alpha = 0.15f) else Color.Transparent)
                .clickable(onClick = onToggleNotes),
        ) {
            Icon(
                painterResource(Res.drawable.ic_sticky_note_2),
                contentDescription = "Toggle Notes Panel",
                tint = if (!isNotesCollapsed) colors.accent else colors.fillSecondary,
                modifier = Modifier.size(16.dp),
            )
        }

        Spacer(Modifier.width(2.dp))

        // Mode switchers
        Chip(stringResource(Res.string.reader_mode_pdf), selected = mode == ReaderMode.Pdf, enabled = hasPdf) {
            onModeChange(ReaderMode.Pdf)
        }
        Chip(stringResource(Res.string.reader_mode_text), selected = mode == ReaderMode.Text) {
            onModeChange(ReaderMode.Text)
        }
        Chip("Split", selected = mode == ReaderMode.Split, enabled = hasPdf) {
            onModeChange(ReaderMode.Split)
        }

        Spacer(Modifier.width(4.dp))

        // PDF Zoom or Text highlight
        when (mode) {
            ReaderMode.Pdf, ReaderMode.Split -> {
                val zoomOut = stringResource(Res.string.reader_zoom_out)
                val zoomIn = stringResource(Res.string.reader_zoom_in)
                TextButton(onClick = onZoomOut, modifier = Modifier.semantics { contentDescription = "$zoomOut (Ctrl+-)" }) {
                    Text("−", style = MaterialTheme.typography.titleLarge)
                }
                TextButton(onClick = onZoomReset) { Text(stringResource(Res.string.reader_zoom_reset) + " (Ctrl+0)") }
                TextButton(onClick = onZoomIn, modifier = Modifier.semantics { contentDescription = "$zoomIn (Ctrl++)" }) {
                    Text("+", style = MaterialTheme.typography.titleLarge)
                }
            }
            ReaderMode.Text -> {
                if (canHighlight) {
                    TextButton(onClick = onHighlight) {
                        Text(stringResource(Res.string.reader_highlight))
                    }
                }
            }
        }

        // Typography & Theme Menu "Aa"
        Box {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                    .background(if (typographyMenuOpen) colors.fillQuarternary else Color.Transparent)
                    .clickable { typographyMenuOpen = true }
                    .padding(horizontal = 8.dp, vertical = 4.dp),
            ) {
                Text(
                    text = "Aa",
                    style = MaterialTheme.typography.labelMedium.copy(
                        fontWeight = FontWeight.Bold,
                        fontFamily = when (readerFont) {
                            ReaderFont.Serif -> FontFamily.Serif
                            ReaderFont.Sans -> FontFamily.Default
                            ReaderFont.Mono -> FontFamily.Monospace
                        },
                    ),
                    color = colors.fillPrimary,
                )
                Spacer(Modifier.width(4.dp))
                // Theme color preview dot
                Box(
                    modifier = Modifier
                        .size(8.dp)
                        .clip(CircleShape)
                        .background(readerColors(readerTheme).background)
                        .border(1.dp, colors.fillTertiary, CircleShape),
                )
            }

            DropdownMenu(
                expanded = typographyMenuOpen,
                onDismissRequest = { typographyMenuOpen = false },
                modifier = Modifier
                    .width(260.dp)
                    .background(colors.sidepane)
                    .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase))
                    .padding(Dimens.SpaceSm),
            ) {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    // Font Family Section
                    Text(
                        "Typography",
                        style = MaterialTheme.typography.labelSmall,
                        color = colors.fillSecondary,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        ReaderFont.entries.forEach { font ->
                            val isSelected = readerFont == font
                            Box(
                                modifier = Modifier
                                    .weight(1f)
                                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                    .background(if (isSelected) colors.accent else colors.background)
                                    .border(1.dp, if (isSelected) colors.accent else colors.border, RoundedCornerShape(Dimens.RadiusSmall))
                                    .clickable { onSetReaderFont(font) }
                                    .padding(vertical = 6.dp),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(
                                    text = font.title,
                                    style = MaterialTheme.typography.labelSmall.copy(
                                        fontFamily = when (font) {
                                            ReaderFont.Serif -> FontFamily.Serif
                                            ReaderFont.Sans -> FontFamily.Default
                                            ReaderFont.Mono -> FontFamily.Monospace
                                        },
                                        fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Normal,
                                    ),
                                    color = if (isSelected) colors.onAccent else colors.fillPrimary,
                                )
                            }
                        }
                    }

                    // Font Size Section
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        Text("Size (Ctrl +/-)", style = MaterialTheme.typography.labelSmall, color = colors.fillSecondary)
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            Box(
                                modifier = Modifier
                                    .size(26.dp)
                                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                    .background(colors.background)
                                    .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusSmall))
                                    .clickable { onChangeFontSize(-1) },
                                contentAlignment = Alignment.Center,
                            ) {
                                Text("A-", style = MaterialTheme.typography.labelSmall, color = colors.fillPrimary)
                            }
                            Text(
                                "${fontSizeSp}px",
                                style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace),
                                color = colors.fillPrimary,
                                modifier = Modifier.padding(horizontal = 4.dp),
                            )
                            Box(
                                modifier = Modifier
                                    .size(26.dp)
                                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                    .background(colors.background)
                                    .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusSmall))
                                    .clickable { onChangeFontSize(1) },
                                contentAlignment = Alignment.Center,
                            ) {
                                Text("A+", style = MaterialTheme.typography.labelSmall, color = colors.fillPrimary)
                            }
                        }
                    }

                    HorizontalDivider(color = colors.fillQuarternary)

                    // Theme Palette Section
                    Text(
                        "Reader Color",
                        style = MaterialTheme.typography.labelSmall,
                        color = colors.fillSecondary,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Row(
                        horizontalArrangement = Arrangement.SpaceBetween,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        ReaderTheme.entries.forEach { theme ->
                            val themeColors = readerColors(theme)
                            val isSelected = readerTheme == theme
                            Box(
                                modifier = Modifier
                                    .size(30.dp)
                                    .clip(CircleShape)
                                    .background(themeColors.background)
                                    .border(
                                        width = if (isSelected) 2.dp else 1.dp,
                                        color = if (isSelected) colors.accent else Color.White.copy(alpha = 0.2f),
                                        shape = CircleShape,
                                    )
                                    .clickable { onSetReaderTheme(theme) },
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(
                                    text = "Aa",
                                    style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp),
                                    color = themeColors.text,
                                )
                            }
                        }
                    }
                }
            }
        }

        // Search trigger button (TUI Parity: in-document search Ctrl+F)
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier
                .size(Dimens.ToolbarButton)
                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                .background(if (isSearchOpen) colors.accent.copy(alpha = 0.15f) else Color.Transparent)
                .clickable(onClick = onToggleSearch),
        ) {
            Icon(
                painterResource(Res.drawable.ic_search),
                contentDescription = "Search In Document (Ctrl+F)",
                tint = if (isSearchOpen) colors.accent else colors.fillSecondary,
                modifier = Modifier.size(16.dp),
            )
        }

        Spacer(Modifier.weight(1f))

        // Reading Status (far right)
        listOf(
            ReadingStatus.Unread to Res.string.reading_unread,
            ReadingStatus.Reading to Res.string.reading_reading,
            ReadingStatus.Read to Res.string.reading_read,
        ).forEach { (status, label) ->
            Chip(stringResource(label), selected = reading == status, status = status) { onReadingChange(status) }
        }
    }
}

@Composable
private fun Chip(text: String, selected: Boolean, enabled: Boolean = true, status: ReadingStatus? = null, onClick: () -> Unit) {
    val colors = QuireTheme.colors
    val selectedBg = when (status) {
        ReadingStatus.Read -> colors.statusRead
        ReadingStatus.Reading -> colors.statusReading
        ReadingStatus.Unread -> colors.fillSecondary
        null -> colors.accent
    }
    Text(
        text,
        style = MaterialTheme.typography.labelSmall,
        color = if (selected) colors.onAccent else colors.fillPrimary,
        modifier = Modifier
            .alpha(if (enabled) 1f else 0.4f)
            .clip(RoundedCornerShape(Dimens.RadiusFull))
            .background(if (selected) selectedBg else colors.fillQuinary)
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = Dimens.SpaceSm, vertical = 5.dp),
    )
}

