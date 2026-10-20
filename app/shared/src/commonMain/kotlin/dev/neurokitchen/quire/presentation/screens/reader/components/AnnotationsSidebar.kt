package dev.neurokitchen.quire.presentation.screens.reader.components

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
import androidx.compose.ui.text.style.TextOverflow
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
import org.jetbrains.compose.resources.stringResource

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
) {
    val colors = QuireTheme.colors
    Column(modifier.fillMaxHeight().background(colors.sidepane)) {
        Text(
            stringResource(Res.string.reader_annotations),
            style = MaterialTheme.typography.titleMedium,
            color = colors.fillPrimary,
            modifier = Modifier.padding(Dimens.SpaceSm),
        )
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
    val shape = RoundedCornerShape(Dimens.RadiusLarge)
    Column(
        Modifier
            .padding(vertical = 4.dp)
            .fillMaxWidth()
            .background(colors.background, shape)
            .border(if (isFocused) 2.dp else 1.dp, if (isFocused) colors.accent else colors.border, shape)
            .clickable(onClick = onClick)
            .padding(Dimens.SpaceXs),
    ) {
        Row {
            Box(Modifier.width(3.dp).height(36.dp).background(HighlightYellow))
            Text(
                highlight.quote,
                style = MaterialTheme.typography.bodyMedium,
                color = colors.fillPrimary,
                maxLines = 4,
                overflow = TextOverflow.Ellipsis,
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
            modifier = Modifier.fillMaxWidth().padding(top = 4.dp),
        )
        Row {
            if (draft != highlight.comment.orEmpty()) TextButton(onClick = { onComment(draft) }) { Text(stringResource(Res.string.reader_save_comment)) }
            TextButton(onClick = onRemove) { Text(stringResource(Res.string.reader_remove)) }
        }
    }
}
