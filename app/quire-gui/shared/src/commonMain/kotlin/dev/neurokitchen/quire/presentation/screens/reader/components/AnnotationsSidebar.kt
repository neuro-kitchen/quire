package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.CompactTextField
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.annotations_collapse_annotations
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.reader_add_comment
import dev.neurokitchen.quire.resources.reader_annotations
import dev.neurokitchen.quire.resources.reader_no_annotations
import dev.neurokitchen.quire.resources.reader_not_found_in_pdf
import dev.neurokitchen.quire.resources.reader_remove
import dev.neurokitchen.quire.resources.reader_save_comment
import org.jetbrains.compose.resources.painterResource
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
    onCollapse: (() -> Unit)? = null,
    /** The "Annotations" heading (not under the phone's Notes tabs, which name it). */
    showTitle: Boolean = true,
) {
    val colors = QuireTheme.colors
    val isMobileSheet = !showTitle
    Column(
        modifier
            .fillMaxHeight()
            .background(if (isMobileSheet) colors.background else colors.sidepane),
    ) {
        if (showTitle) Row(
            verticalAlignment = Alignment.CenterVertically,
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
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier.fillMaxSize().padding(Dimens.SpaceLg),
            ) {
                Text(
                    stringResource(Res.string.reader_no_annotations),
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.fillSecondary,
                    textAlign = TextAlign.Center,
                )
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                contentPadding = if (isMobileSheet) {
                    PaddingValues(horizontal = Dimens.SpaceLg, vertical = Dimens.SpaceSm)
                } else {
                    PaddingValues(horizontal = Dimens.SpaceXs, vertical = Dimens.SpaceMin)
                },
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                items(highlights, key = { it.id }) { highlight ->
                    Annotation(
                        highlight = highlight,
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
    var isEditingComment by remember(highlight.id) { mutableStateOf(false) }
    val hasSavedComment = !highlight.comment.isNullOrBlank()

    val barColor = when (highlight.color.lowercase()) {
        "green" -> colors.highlightGreen
        "blue" -> colors.highlightBlue
        "purple" -> colors.highlightPurple
        "red" -> colors.highlightRed
        else -> colors.highlightYellow
    }
    val shape = RoundedCornerShape(Dimens.RadiusLarge)
    Column(
        Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(
                if (isFocused) colors.accent.copy(alpha = 0.10f)
                else colors.cardBackground,
            )
            .clickable(onClick = onClick)
            .padding(14.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        // The colour bar runs the passage's height.
        Row(Modifier.height(IntrinsicSize.Min)) {
            Box(Modifier.width(4.dp).fillMaxHeight().background(barColor, RoundedCornerShape(2.dp)))
            Text(
                highlight.quote,
                style = MaterialTheme.typography.bodyMedium.copy(lineHeight = 21.sp),
                color = colors.fillPrimary,
                modifier = Modifier.padding(start = Dimens.SpaceSm),
            )
        }

        if (hasSavedComment && !isEditingComment) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(Dimens.RadiusBase))
                    .background(colors.fillQuinary)
                    .clickable { isEditingComment = true }
                    .padding(horizontal = 10.dp, vertical = 8.dp),
            ) {
                Text(
                    text = highlight.comment.orEmpty(),
                    style = MaterialTheme.typography.bodySmall.copy(fontStyle = FontStyle.Italic),
                    color = colors.fillSecondary,
                )
            }
        }

        if (isEditingComment) {
            CompactTextField(
                value = draft,
                onValueChange = { draft = it },
                placeholder = stringResource(Res.string.reader_add_comment),
                modifier = Modifier.fillMaxWidth(),
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(colors.accent)
                            .clickable {
                                onComment(draft.trim())
                                isEditingComment = false
                            }
                            .padding(horizontal = 12.dp, vertical = 5.dp),
                    ) {
                        Text(
                            text = stringResource(Res.string.reader_save_comment),
                            style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.SemiBold),
                            color = colors.onAccent,
                        )
                    }
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(colors.fillQuinary)
                            .clickable {
                                draft = highlight.comment.orEmpty()
                                isEditingComment = false
                            }
                            .padding(horizontal = 12.dp, vertical = 5.dp),
                    ) {
                        Text(
                            text = "Cancel",
                            style = MaterialTheme.typography.labelMedium,
                            color = colors.fillSecondary,
                        )
                    }
                }
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .clickable(onClick = onRemove)
                        .padding(horizontal = 8.dp, vertical = 4.dp),
                ) {
                    Text(
                        text = stringResource(Res.string.reader_remove),
                        style = MaterialTheme.typography.labelMedium,
                        color = colors.fillTertiary,
                    )
                }
            }
        } else {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
                ) {
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(colors.fillQuinary)
                            .clickable { isEditingComment = true }
                            .padding(horizontal = 10.dp, vertical = 4.dp),
                    ) {
                        Text(
                            text = if (hasSavedComment) "Edit comment" else stringResource(Res.string.reader_add_comment),
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Medium),
                            color = colors.fillSecondary,
                        )
                    }
                    if (isMissing) {
                        Text(
                            text = stringResource(Res.string.reader_not_found_in_pdf),
                            style = MaterialTheme.typography.labelSmall,
                            color = colors.fillTertiary,
                        )
                    }
                }
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .clickable(onClick = onRemove)
                        .padding(horizontal = 8.dp, vertical = 4.dp),
                ) {
                    Text(
                        text = stringResource(Res.string.reader_remove),
                        style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Medium),
                        color = colors.fillTertiary,
                    )
                }
            }
        }
    }
}
