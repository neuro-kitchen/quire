package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import dev.neurokitchen.quire.domain.model.PdfFigure
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.reader.Pinned
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.figure_unavailable
import dev.neurokitchen.quire.resources.reader_close_pane
import org.jetbrains.compose.resources.stringResource

/**
 * A figure or table in its own pane, beside the text or the PDF: the figure as wide as the pane,
 * its caption under it; a table as a table. Scrolls when taller than the pane.
 */
@Composable
fun PinnedPane(
    pinned: Pinned,
    loadFigure: suspend (String) -> String?,
    loadPdfFigure: suspend (PdfFigure) -> ImageBitmap?,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    val title = when (pinned) {
        is Pinned.Figure -> pinned.figure.title
        is Pinned.Table -> pinned.table.title
    }
    Column(modifier.fillMaxSize().background(colors.sidepane)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
            modifier = Modifier.fillMaxWidth().padding(start = Dimens.SpaceMd, end = Dimens.SpaceXs),
        ) {
            Text(title, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.SemiBold, color = colors.fillPrimary, maxLines = 1, modifier = Modifier.weight(1f))
            TextButton(onClick = onClose) { Text(stringResource(Res.string.reader_close_pane)) }
        }
        Column(
            Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(Dimens.SpaceMd),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
        ) {
            when (pinned) {
                is Pinned.Figure -> {
                    val figure = pinned.figure
                    val image: ImageBitmap? by produceState<ImageBitmap?>(null, figure) { value = figureImage(figure, loadFigure, loadPdfFigure) }
                    image?.let { Image(it, contentDescription = figure.title, contentScale = ContentScale.FillWidth, modifier = Modifier.fillMaxWidth()) }
                        ?: Text(stringResource(Res.string.figure_unavailable), style = MaterialTheme.typography.bodySmall, color = colors.fillTertiary)
                    if (figure.caption.isNotBlank()) {
                        SelectionContainer {
                            Text(figure.caption, style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary)
                        }
                    }
                }
                is Pinned.Table -> PaperTable(
                    rows = pinned.table.rows,
                    textStyle = MaterialTheme.typography.bodyMedium,
                    textColor = colors.fillPrimary,
                    lineColor = colors.border.copy(alpha = 0.5f),
                    stripeColor = colors.background.copy(alpha = 0.6f),
                )
            }
        }
    }
}
