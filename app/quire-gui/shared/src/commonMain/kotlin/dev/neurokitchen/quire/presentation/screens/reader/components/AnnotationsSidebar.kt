package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.layout.IntrinsicSize
import dev.neurokitchen.quire.resources.annotations_collapse_annotations
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.CompactTextField
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.reader_add_comment
import dev.neurokitchen.quire.resources.reader_annotations
import dev.neurokitchen.quire.resources.reader_no_annotations
import dev.neurokitchen.quire.resources.reader_not_found_in_pdf
import dev.neurokitchen.quire.resources.reader_remove
import dev.neurokitchen.quire.resources.reader_save_comment
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import dev.neurokitchen.quire.resources.ic_close

/** The paper's highlights, like Zotero's annotation sidebar: click one to bring it into view. */
@Composable
fun AnnotationsSidebar(
    highlights: List<Highlight>,
    focused: String?,
    /** Highlights not found on the PDF's pages (only in PDF mode). */
    missing: Set<String>,
    onClick: (String) -> Unit,
    onComment: (String, String) -> Unit,
    onRemove: (String) -> Unit,
    modifier: Modifier = Modifier,
    onCollapse: (() -> Unit)? = null,
    /** The "Annotations" heading (not under the phone's Notes tabs, which name it). */
    showTitle: Boolean = true,
) {
    val colors = QuireTheme.colors
    Column(modifier.fillMaxHeight().background(colors.sidepane)) {
        if (showTitle) Row(
            verticalAlignment = androidx.compose.ui.Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceSm, vertical = 6.dp),
        ) {
            Text(
                stringResource(Res.string.reader_annotations),
                style = MaterialTheme.typography.titleMedium,
                color = colors.fillPrimary,
            )
            Spacer(Modifier.weight(1f))
            if (onCollapse != null) {
                IconButton(onClick = onCollapse, modifier = Modifier.size(24.dp)) {
                    Icon(
                        painterResource(Res.drawable.ic_close),
                        contentDescription = stringResource(Res.string.annotations_collapse_annotations),
                        tint = colors.fillSecondary,
                        modifier = Modifier.size(16.dp),
                    )
                }
            }
        }
        if (highlights.isEmpty()) {
            Text(
                stringResource(Res.string.reader_no_annotations),
                style = MaterialTheme.typography.bodyMedium,
                color = colors.fillSecondary,
                modifier = Modifier.padding(horizontal = Dimens.SpaceSm),
            )
        }
        LazyColumn(Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceXs)) {
            items(highlights, key = { it.id }) { highlight ->
                Annotation(
                    highlight,
                    isFocused = highlight.id == focused,
                    isMissing = highlight.id in missing,
                    onClick = { onClick(highlight.id) },
                    onComment = { onComment(highlight.id, it) },
                    onRemove = { onRemove(highlight.id) },
                )
            }
        }
    }
}

/** The highlight color (Zotero's default yellow). */
val HighlightYellow = Color(0xFFFFD400)

@Composable
private fun Annotation(
    highlight: Highlight,
    isFocused: Boolean,
    isMissing: Boolean,
    onClick: () -> Unit,
    onComment: (String) -> Unit,
    onRemove: () -> Unit,
) {
    val colors = QuireTheme.colors
    var draft by remember(highlight.id, highlight.comment) { mutableStateOf(highlight.comment.orEmpty()) }
    val barColor = when (highlight.color.lowercase()) {
        "green" -> colors.highlightGreen
        "blue" -> colors.highlightBlue
        "purple" -> colors.highlightPurple
        "red" -> colors.highlightRed
        else -> colors.highlightYellow
    }
    val shape = RoundedCornerShape(Dimens.RadiusBase)
    Column(
        Modifier
            .padding(vertical = Dimens.SpaceMin)
            .fillMaxWidth()
            .background(colors.background, shape)
            .border(if (isFocused) 2.dp else 1.dp, if (isFocused) colors.accent else colors.border, shape)
            .clickable(onClick = onClick)
            .padding(Dimens.SpaceXs),
    ) {
        // The colour bar runs the passage's height.
        Row(Modifier.height(IntrinsicSize.Min)) {
            Box(Modifier.width(4.dp).fillMaxHeight().background(barColor, RoundedCornerShape(2.dp)))
            Text(
                highlight.quote,
                style = MaterialTheme.typography.bodyMedium,
                color = colors.fillPrimary,
                // The whole highlighted passage (it was cut after four lines).
                modifier = Modifier.padding(start = Dimens.SpaceXs),
            )
        }
        if (isMissing) {
            Text(stringResource(Res.string.reader_not_found_in_pdf), style = MaterialTheme.typography.labelSmall, color = colors.fillTertiary)
        }
        CompactTextField(
            value = draft,
            onValueChange = { draft = it },
            placeholder = stringResource(Res.string.reader_add_comment),
            modifier = Modifier.fillMaxWidth().padding(top = Dimens.SpaceMin),
        )
        Row {
            if (draft != highlight.comment.orEmpty()) TextButton(onClick = { onComment(draft) }) { Text(stringResource(Res.string.reader_save_comment)) }
            TextButton(onClick = onRemove) { Text(stringResource(Res.string.reader_remove)) }
        }
    }
}
