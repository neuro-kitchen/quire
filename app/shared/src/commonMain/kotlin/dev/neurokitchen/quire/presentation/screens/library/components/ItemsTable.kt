package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
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
import dev.neurokitchen.quire.resources.ic_expand_more
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * Zotero's items list: a sortable header and striped rows (Title, Creator, Year, attachment).
 * Virtualized by `LazyColumn`, so it stays fast with large libraries.
 */
@Composable
fun ItemsTable(
    references: List<ReferenceCardState>,
    sort: SortColumn,
    ascending: Boolean,
    onSortClick: (SortColumn) -> Unit,
    onReferenceClick: (String) -> Unit,
    modifier: Modifier = Modifier,
    /** Double click: open in the reader. */
    onReferenceOpen: (String) -> Unit = {},
) {
    val colors = QuireTheme.colors
    Column(modifier.fillMaxSize().background(colors.background)) {
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
        LazyColumn(Modifier.fillMaxSize()) {
            itemsIndexed(references, key = { _, it -> it.id }) { index, reference ->
                ItemRow(
                    reference,
                    striped = index % 2 == 1,
                    onClick = { onReferenceClick(reference.id) },
                    onDoubleClick = { onReferenceOpen(reference.id) },
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
private fun ItemRow(reference: ReferenceCardState, striped: Boolean, onClick: () -> Unit, onDoubleClick: () -> Unit) {
    val colors = QuireTheme.colors
    val content = if (reference.isSelected) colors.onAccent else colors.fillPrimary
    val background = when {
        reference.isSelected -> colors.accent
        striped -> colors.stripe
        else -> Color.Transparent
    }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .height(Dimens.RowHeight)
            .padding(horizontal = Dimens.SpaceXs)
            .background(background, MaterialTheme.shapes.extraSmall)
            .combinedClickable(onClick = onClick, onDoubleClick = onDoubleClick),
    ) {
        Row(Modifier.weight(4f).padding(horizontal = Dimens.SpaceXs), verticalAlignment = Alignment.CenterVertically) {
            Icon(
                painterResource(kindIcon(reference.kind)), null,
                tint = if (reference.isSelected) colors.onAccent else colors.fillSecondary,
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
                    tint = if (reference.isSelected) colors.onAccent else colors.accentRed,
                    modifier = Modifier.size(Dimens.Icon),
                )
            }
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

