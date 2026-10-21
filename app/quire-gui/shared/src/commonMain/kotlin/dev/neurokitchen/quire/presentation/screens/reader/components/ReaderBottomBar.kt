package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.ReaderFont
import dev.neurokitchen.quire.presentation.screens.reader.ReaderMode
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_expand_more
import dev.neurokitchen.quire.resources.ic_inventory_2
import dev.neurokitchen.quire.resources.ic_sticky_note_2
import dev.neurokitchen.quire.resources.reading_read
import dev.neurokitchen.quire.resources.reading_reading
import dev.neurokitchen.quire.resources.reading_unread
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * Kindle-style expanding reader bottom bar:
 * - Collapsed: Slim 30dp bar with a top accent progress track, document title,
 *   page progress indicator ("Page X of Y • Z% read"), and mode/zoom indicator.
 * - Expanded: Slides up on click to reveal document details, reading progress scrubber,
 *   metadata statistics (notes, highlights, pages), and quick panel toggles.
 */
@Composable
fun ReaderBottomBar(
    title: String,
    current: Int,
    total: Int,
    mode: ReaderMode,
    reading: ReadingStatus,
    zoomPercent: Int = 100,
    fontSizeSp: Int = 16,
    readerFont: ReaderFont = ReaderFont.Serif,
    highlightsCount: Int = 0,
    notesCount: Int = 0,
    source: String? = null,
    onToggleNotes: () -> Unit = {},
    onToggleFigures: () -> Unit = {},
    onToggleAnnotations: () -> Unit = {},
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    var isExpanded by remember { mutableStateOf(false) }

    val safeTotal = total.coerceAtLeast(1)
    val safeCurrent = current.coerceIn(1, safeTotal)
    val progressFraction = if (total > 0) (safeCurrent.toFloat() / safeTotal.toFloat()).coerceIn(0f, 1f) else 0f
    val progressPercent = (progressFraction * 100).toInt()

    Column(
        modifier = modifier
            .fillMaxWidth()
            .background(colors.toolbar)
            .border(width = 1.dp, color = colors.border.copy(alpha = 0.5f)),
    ) {
        // Expanded Panel (Kindle Book Details & Quick Actions Drawer)
        AnimatedVisibility(
            visible = isExpanded,
            enter = expandVertically() + fadeIn(),
            exit = shrinkVertically() + fadeOut(),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(colors.sidepane)
                    .padding(horizontal = 16.dp, vertical = 12.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                // Expanded Header
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        // Reading Status Badge
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusFull))
                                .background(
                                    when (reading) {
                                        ReadingStatus.Read -> colors.statusRead.copy(alpha = 0.18f)
                                        ReadingStatus.Reading -> colors.statusReading.copy(alpha = 0.18f)
                                        ReadingStatus.Unread -> colors.fillQuinary
                                    }
                                )
                                .padding(horizontal = 8.dp, vertical = 3.dp),
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(7.dp)
                                    .clip(CircleShape)
                                    .background(
                                        when (reading) {
                                            ReadingStatus.Read -> colors.statusRead
                                            ReadingStatus.Reading -> colors.statusReading
                                            ReadingStatus.Unread -> colors.fillSecondary
                                        }
                                    )
                            )
                            Spacer(Modifier.width(5.dp))
                            Text(
                                text = when (reading) {
                                    ReadingStatus.Read -> stringResource(Res.string.reading_read)
                                    ReadingStatus.Reading -> stringResource(Res.string.reading_reading)
                                    ReadingStatus.Unread -> stringResource(Res.string.reading_unread)
                                },
                                style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp, fontWeight = FontWeight.Medium),
                                color = colors.fillPrimary,
                            )
                        }

                        // Mode Badge
                        Text(
                            text = if (mode == ReaderMode.Pdf) "PDF View" else "Reader Mode",
                            style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp),
                            color = colors.fillTertiary,
                        )
                    }

                    // Close Drawer Button
                    Box(
                        contentAlignment = Alignment.Center,
                        modifier = Modifier
                            .size(22.dp)
                            .clip(CircleShape)
                            .background(colors.fillQuarternary.copy(alpha = 0.6f))
                            .clickable { isExpanded = false },
                    ) {
                        Icon(
                            painterResource(Res.drawable.ic_close),
                            contentDescription = "Collapse Bottom Bar",
                            tint = colors.fillSecondary,
                            modifier = Modifier.size(13.dp),
                        )
                    }
                }

                // Full Paper Title
                Text(
                    text = title.ifEmpty { "Untitled Document" },
                    style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Bold, lineHeight = 18.sp),
                    color = colors.fillPrimary,
                )

                // Large Kindle Reading Progress Bar
                Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = if (mode == ReaderMode.Pdf) "Page $safeCurrent of $safeTotal" else "Section $safeCurrent of $safeTotal",
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.SemiBold),
                            color = colors.fillPrimary,
                        )
                        Text(
                            text = "$progressPercent% completed",
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.SemiBold),
                            color = colors.accent,
                        )
                    }

                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(6.dp)
                            .clip(RoundedCornerShape(3.dp))
                            .background(colors.fillQuarternary),
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(progressFraction)
                                .fillMaxHeight()
                                .clip(RoundedCornerShape(3.dp))
                                .background(colors.accent),
                        )
                    }
                }

                // Metadata Statistics & Quick Toggles
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        MetaStat(label = "Highlights", value = highlightsCount.toString())
                        MetaStat(label = "Notes", value = notesCount.toString())
                        if (source != null) {
                            MetaStat(label = "Source", value = source)
                        }
                    }

                    // Quick Panel Navigation Shortcuts
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        QuickActionChip("Annotations", Res.drawable.ic_inventory_2, onToggleAnnotations)
                        QuickActionChip("Notes", Res.drawable.ic_sticky_note_2, onToggleNotes)
                        QuickActionChip("Figures", Res.drawable.ic_auto_fix_high, onToggleFigures)
                    }
                }

                HorizontalDivider(color = colors.border.copy(alpha = 0.3f), modifier = Modifier.padding(top = 2.dp))
            }
        }

        // Hairline Reading Progress Accent Track (Always visible at top of bottom bar)
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(2.5.dp)
                .background(colors.border.copy(alpha = 0.25f)),
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth(progressFraction)
                    .fillMaxHeight()
                    .background(colors.accent),
            )
        }

        // Redesigned Modern Reader Control Bar (54dp + navigationBarsPadding)
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding(),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(54.dp)
                    .padding(horizontal = 12.dp),
            ) {
            // Left: Interactive Reading Progress Pill (Click to toggle details drawer)
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .clip(RoundedCornerShape(Dimens.RadiusFull))
                    .background(colors.fillQuinary)
                    .border(1.dp, colors.border.copy(alpha = 0.5f), RoundedCornerShape(Dimens.RadiusFull))
                    .clickable { isExpanded = !isExpanded }
                    .padding(horizontal = 10.dp, vertical = 6.dp),
            ) {
                Box(
                    modifier = Modifier
                        .size(7.dp)
                        .clip(CircleShape)
                        .background(
                            when (reading) {
                                ReadingStatus.Read -> colors.statusRead
                                ReadingStatus.Reading -> colors.statusReading
                                ReadingStatus.Unread -> colors.fillSecondary
                            }
                        )
                )
                Spacer(Modifier.width(6.dp))
                Text(
                    text = if (total > 0) {
                        if (mode == ReaderMode.Pdf) "p. $safeCurrent/$safeTotal • $progressPercent%"
                        else "pt. $safeCurrent/$safeTotal • $progressPercent%"
                    } else "Reading",
                    style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp, fontWeight = FontWeight.SemiBold),
                    color = colors.fillPrimary,
                )
                Spacer(Modifier.width(4.dp))
                Icon(
                    painterResource(Res.drawable.ic_expand_more),
                    contentDescription = if (isExpanded) "Collapse Details" else "Expand Details",
                    tint = colors.fillSecondary,
                    modifier = Modifier
                        .size(14.dp)
                        .rotate(if (isExpanded) 180f else 0f),
                )
            }

            // Center: Document Title
            Text(
                text = title.ifEmpty { "Document" },
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Medium),
                color = colors.fillSecondary,
                modifier = Modifier
                    .weight(1f, fill = false)
                    .padding(horizontal = 8.dp),
            )

            // Right: Reader Panel Triggers (Highlights, Notes, Figures)
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                // Annotations Button with Badge
                Box(
                    contentAlignment = Alignment.Center,
                    modifier = Modifier
                        .height(32.dp)
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .background(if (highlightsCount > 0) colors.accent.copy(alpha = 0.12f) else colors.fillQuinary)
                        .border(1.dp, if (highlightsCount > 0) colors.accent.copy(alpha = 0.4f) else colors.border.copy(alpha = 0.4f), RoundedCornerShape(Dimens.RadiusSmall))
                        .clickable(onClick = onToggleAnnotations)
                        .padding(horizontal = 8.dp),
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                        Icon(
                            painterResource(Res.drawable.ic_inventory_2),
                            contentDescription = "Annotations",
                            tint = if (highlightsCount > 0) colors.accent else colors.fillSecondary,
                            modifier = Modifier.size(15.dp),
                        )
                        if (highlightsCount > 0) {
                            Text(
                                text = "$highlightsCount",
                                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold, fontSize = 11.sp),
                                color = colors.accent,
                            )
                        }
                    }
                }

                // Notes Button with Badge
                Box(
                    contentAlignment = Alignment.Center,
                    modifier = Modifier
                        .height(32.dp)
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .background(if (notesCount > 0) colors.accent.copy(alpha = 0.12f) else colors.fillQuinary)
                        .border(1.dp, if (notesCount > 0) colors.accent.copy(alpha = 0.4f) else colors.border.copy(alpha = 0.4f), RoundedCornerShape(Dimens.RadiusSmall))
                        .clickable(onClick = onToggleNotes)
                        .padding(horizontal = 8.dp),
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                        Icon(
                            painterResource(Res.drawable.ic_sticky_note_2),
                            contentDescription = "Notes",
                            tint = if (notesCount > 0) colors.accent else colors.fillSecondary,
                            modifier = Modifier.size(15.dp),
                        )
                        if (notesCount > 0) {
                            Text(
                                text = "$notesCount",
                                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold, fontSize = 11.sp),
                                color = colors.accent,
                            )
                        }
                    }
                }

                // Figures Drawer Button
                Box(
                    contentAlignment = Alignment.Center,
                    modifier = Modifier
                        .size(32.dp)
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .background(colors.fillQuinary)
                        .border(1.dp, colors.border.copy(alpha = 0.4f), RoundedCornerShape(Dimens.RadiusSmall))
                        .clickable(onClick = onToggleFigures),
                ) {
                    Icon(
                        painterResource(Res.drawable.ic_auto_fix_high),
                        contentDescription = "Figures & Tables",
                        tint = colors.fillSecondary,
                        modifier = Modifier.size(15.dp),
                    )
                }
            }
        }
    }
}
}

@Composable
private fun MetaStat(label: String, value: String) {
    val colors = QuireTheme.colors
    Row(verticalAlignment = Alignment.CenterVertically) {
        Text(
            text = "$label: ",
            style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
            color = colors.fillTertiary,
        )
        Text(
            text = value,
            style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp, fontWeight = FontWeight.SemiBold),
            color = colors.fillPrimary,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun QuickActionChip(
    label: String,
    icon: org.jetbrains.compose.resources.DrawableResource,
    onClick: () -> Unit,
) {
    val colors = QuireTheme.colors
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .height(24.dp)
            .clip(RoundedCornerShape(Dimens.RadiusSmall))
            .background(colors.fillQuarternary.copy(alpha = 0.5f))
            .clickable(onClick = onClick)
            .padding(horizontal = 7.dp),
    ) {
        Icon(
            painterResource(icon),
            contentDescription = null,
            tint = colors.fillSecondary,
            modifier = Modifier.size(12.dp),
        )
        Spacer(Modifier.width(4.dp))
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.Medium),
            color = colors.fillPrimary,
        )
    }
}
