package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.isCtrlPressed
import androidx.compose.ui.input.pointer.isMetaPressed
import androidx.compose.ui.input.pointer.isSecondaryPressed
import androidx.compose.ui.input.pointer.isShiftPressed
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.SortColumn
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mapping.kindIcon
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.column_attachments
import dev.neurokitchen.quire.resources.column_creator
import dev.neurokitchen.quire.resources.column_title
import dev.neurokitchen.quire.resources.column_year
import dev.neurokitchen.quire.resources.ic_attach_file
import dev.neurokitchen.quire.resources.ic_content_copy
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_expand_more
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * Items list with:
 * - Sortable columns
 * - Multi-selection (Shift+Click range, Ctrl+Click toggle)
 * - Right-click context menu (Open in Reader, Add to Collection, Copy Citation, Delete, Status)
 * - Batch selection action banner
 * - Virtualized by LazyColumn
 */
@Composable
fun ItemsTable(
    references: List<ReferenceCardState>,
    sort: SortColumn,
    ascending: Boolean,
    selectedIds: Set<String> = emptySet(),
    onSortClick: (SortColumn) -> Unit,
    onReferenceClick: (String) -> Unit,
    onReferenceSelect: (id: String, isCtrl: Boolean, isShift: Boolean) -> Unit = { id, _, _ -> onReferenceClick(id) },
    onReferenceOpen: (String) -> Unit = {},
    onDeleteReference: (String) -> Unit = {},
    onDeleteSelected: () -> Unit = {},
    onSelectAll: () -> Unit = {},
    onClearSelection: () -> Unit = {},
    onCreateCollectionFromSelected: () -> Unit = {},
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors

    Column(modifier.fillMaxSize().background(colors.background)) {
        // Table Header
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().height(Dimens.HeaderHeight).padding(horizontal = Dimens.SpaceXs),
        ) {
            HeaderCell(stringResource(Res.string.column_title), SortColumn.Title, sort, ascending, onSortClick, Modifier.weight(4f))
            HeaderCell(stringResource(Res.string.column_creator), SortColumn.Creator, sort, ascending, onSortClick, Modifier.weight(1.4f))
            HeaderCell(stringResource(Res.string.column_year), SortColumn.Year, sort, ascending, onSortClick, Modifier.width(Dimens.YearColumnWidth))
            Box(Modifier.width(Dimens.AttachmentColumnWidth), contentAlignment = Alignment.Center) {
                Icon(
                    painterResource(Res.drawable.ic_attach_file), stringResource(Res.string.column_attachments),
                    tint = colors.fillSecondary, modifier = Modifier.size(14.dp),
                )
            }
        }
        HorizontalDivider(color = colors.fillQuarternary)

        // Multi-Selection Action Banner (visible when 2+ papers selected)
        if (selectedIds.size > 1) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .background(colors.accent.copy(alpha = 0.12f))
                    .padding(horizontal = Dimens.SpaceMd, vertical = 6.dp),
            ) {
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .background(colors.accent)
                        .padding(horizontal = 8.dp, vertical = 2.dp),
                ) {
                    Text(
                        "${selectedIds.size} selected",
                        style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                        color = colors.onAccent,
                    )
                }

                TextButton(onClick = onSelectAll) {
                    Text("Select All", style = MaterialTheme.typography.labelSmall, color = colors.accent)
                }

                TextButton(onClick = onClearSelection) {
                    Text("Clear Selection", style = MaterialTheme.typography.labelSmall, color = colors.fillSecondary)
                }

                Spacer(Modifier.weight(1f))

                // Create Collection from Selection button
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .background(colors.fillQuarternary)
                        .clickable(onClick = onCreateCollectionFromSelected)
                        .padding(horizontal = 8.dp, vertical = 4.dp),
                ) {
                    Icon(painterResource(Res.drawable.ic_create_new_folder), contentDescription = null, tint = colors.fillPrimary, modifier = Modifier.size(14.dp))
                    Spacer(Modifier.width(4.dp))
                    Text("New Collection...", style = MaterialTheme.typography.labelSmall, color = colors.fillPrimary)
                }

                // Delete Selected button
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusSmall))
                        .background(colors.accentRed.copy(alpha = 0.15f))
                        .clickable(onClick = onDeleteSelected)
                        .padding(horizontal = 8.dp, vertical = 4.dp),
                ) {
                    Icon(painterResource(Res.drawable.ic_delete), contentDescription = null, tint = colors.accentRed, modifier = Modifier.size(14.dp))
                    Spacer(Modifier.width(4.dp))
                    Text("Delete (${selectedIds.size})", style = MaterialTheme.typography.labelSmall, color = colors.accentRed)
                }
            }
            HorizontalDivider(color = colors.accent.copy(alpha = 0.3f))
        }

        // Table Rows
        LazyColumn(Modifier.fillMaxSize()) {
            itemsIndexed(references, key = { _, it -> it.id }) { index, reference ->
                val isSelected = reference.isSelected || reference.id in selectedIds
                ItemRow(
                    reference = reference,
                    isSelected = isSelected,
                    isMultiSelectActive = selectedIds.size > 1,
                    striped = index % 2 == 1,
                    onSelect = onReferenceSelect,
                    onDoubleClick = { onReferenceOpen(reference.id) },
                    onDelete = { onDeleteReference(reference.id) },
                    onCreateCollection = onCreateCollectionFromSelected,
                )
            }
        }
    }
}

@Composable
private fun HeaderCell(
    label: String,
    column: SortColumn,
    sort: SortColumn,
    ascending: Boolean,
    onSortClick: (SortColumn) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier.fillMaxSize().clickable { onSortClick(column) }.padding(horizontal = Dimens.SpaceXs),
    ) {
        Text(
            label, style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary,
            maxLines = 1, modifier = Modifier.weight(1f, fill = false),
        )
        if (column == sort) {
            Icon(
                painterResource(Res.drawable.ic_expand_more), null, tint = colors.fillSecondary,
                modifier = Modifier.padding(start = 2.dp).size(14.dp).rotate(if (ascending) 180f else 0f),
            )
        }
    }
}

@Composable
private fun ItemRow(
    reference: ReferenceCardState,
    isSelected: Boolean,
    isMultiSelectActive: Boolean,
    striped: Boolean,
    onSelect: (id: String, isCtrl: Boolean, isShift: Boolean) -> Unit,
    onDoubleClick: () -> Unit,
    onDelete: () -> Unit,
    onCreateCollection: () -> Unit,
) {
    val colors = QuireTheme.colors
    val clipboardManager = LocalClipboardManager.current
    var contextMenuOpen by remember { mutableStateOf(false) }

    val content = if (isSelected) colors.onAccent else colors.fillPrimary
    val targetBg = when {
        isSelected -> if (isMultiSelectActive) colors.accent.copy(alpha = 0.85f) else colors.accent
        striped -> colors.stripe
        else -> Color.Transparent
    }
    val background by animateColorAsState(targetValue = targetBg, animationSpec = tween(150))

    Box {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .fillMaxWidth()
                .height(Dimens.RowHeight)
                .padding(horizontal = Dimens.SpaceXs)
                .clip(RoundedCornerShape(Dimens.RadiusSmall))
                .background(background)
                .pointerInput(reference.id) {
                    awaitPointerEventScope {
                        while (true) {
                            val event = awaitPointerEvent()
                            if (event.type == PointerEventType.Press) {
                                val isSecondary = event.buttons.isSecondaryPressed
                                val isShift = event.keyboardModifiers.isShiftPressed
                                val isCtrl = event.keyboardModifiers.isCtrlPressed || event.keyboardModifiers.isMetaPressed
                                if (isSecondary) {
                                    onSelect(reference.id, isCtrl, isShift)
                                    contextMenuOpen = true
                                } else if (isShift || isCtrl) {
                                    onSelect(reference.id, isCtrl, isShift)
                                }
                            }
                        }
                    }
                }
                .combinedClickable(
                    onClick = { onSelect(reference.id, false, false) },
                    onDoubleClick = onDoubleClick,
                ),
        ) {
            Row(Modifier.weight(4f).padding(horizontal = Dimens.SpaceXs), verticalAlignment = Alignment.CenterVertically) {
                // Reading status dot
                val dotColor = when (reference.reading) {
                    ReadingStatus.Read -> colors.statusRead
                    ReadingStatus.Reading -> colors.statusReading
                    ReadingStatus.Unread -> colors.statusUnread.copy(alpha = 0.5f)
                }
                Box(
                    modifier = Modifier
                        .size(6.dp)
                        .clip(CircleShape)
                        .background(if (isSelected) colors.onAccent else dotColor),
                )
                Spacer(Modifier.width(6.dp))
                Icon(
                    painterResource(kindIcon(reference.kind)), null,
                    tint = if (isSelected) colors.onAccent else colors.fillSecondary,
                    modifier = Modifier.size(Dimens.Icon),
                )
                Cell(reference.title, content, Modifier.padding(start = 6.dp))
            }
            Cell(reference.creator, content, Modifier.weight(1.4f).padding(horizontal = Dimens.SpaceXs))
            Cell(reference.year, content, Modifier.width(Dimens.YearColumnWidth).padding(horizontal = Dimens.SpaceXs))
            Box(Modifier.width(Dimens.AttachmentColumnWidth), contentAlignment = Alignment.Center) {
                if (reference.hasPdf) {
                    Icon(
                        painterResource(Res.drawable.ic_picture_as_pdf), null,
                        tint = if (isSelected) colors.onAccent else colors.accentRed,
                        modifier = Modifier.size(Dimens.Icon),
                    )
                }
            }
        }

        // Right-Click Context Menu
        DropdownMenu(
            expanded = contextMenuOpen,
            onDismissRequest = { contextMenuOpen = false },
            modifier = Modifier
                .width(220.dp)
                .background(colors.sidepane)
                .border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)),
        ) {
            DropdownMenuItem(
                text = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(painterResource(Res.drawable.ic_description), contentDescription = null, tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(8.dp))
                        Text("Open in Reader", style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                    }
                },
                onClick = {
                    contextMenuOpen = false
                    onDoubleClick()
                },
            )

            DropdownMenuItem(
                text = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(painterResource(Res.drawable.ic_create_new_folder), contentDescription = null, tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(8.dp))
                        Text("Create Collection...", style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                    }
                },
                onClick = {
                    contextMenuOpen = false
                    onCreateCollection()
                },
            )

            HorizontalDivider(color = colors.fillQuarternary, modifier = Modifier.padding(vertical = 4.dp))

            DropdownMenuItem(
                text = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(painterResource(Res.drawable.ic_content_copy), contentDescription = null, tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(8.dp))
                        Text("Copy Citation", style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                    }
                },
                onClick = {
                    clipboardManager.setText(AnnotatedString("${reference.creator} (${reference.year}). ${reference.title}."))
                    contextMenuOpen = false
                },
            )

            DropdownMenuItem(
                text = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(painterResource(Res.drawable.ic_content_copy), contentDescription = null, tint = colors.fillSecondary, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(8.dp))
                        Text("Copy Title", style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                    }
                },
                onClick = {
                    clipboardManager.setText(AnnotatedString(reference.title))
                    contextMenuOpen = false
                },
            )

            HorizontalDivider(color = colors.fillQuarternary, modifier = Modifier.padding(vertical = 4.dp))

            DropdownMenuItem(
                text = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(painterResource(Res.drawable.ic_delete), contentDescription = null, tint = colors.accentRed, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(8.dp))
                        Text("Delete Paper", style = MaterialTheme.typography.bodyMedium, color = colors.accentRed)
                        Spacer(Modifier.weight(1f))
                        Text("Del", style = MaterialTheme.typography.labelSmall, color = colors.fillTertiary)
                    }
                },
                onClick = {
                    contextMenuOpen = false
                    onDelete()
                },
            )
        }
    }
}

@Composable
private fun RowScope.Cell(text: String, color: Color, modifier: Modifier = Modifier) {
    Text(
        text, style = MaterialTheme.typography.bodyMedium, color = color,
        maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = modifier,
    )
}


