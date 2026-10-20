package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.reader.ReaderMode
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.reader_highlight
import dev.neurokitchen.quire.resources.reader_mode_pdf
import dev.neurokitchen.quire.resources.reader_mode_text
import dev.neurokitchen.quire.resources.reader_zoom_in
import dev.neurokitchen.quire.resources.reader_zoom_out
import dev.neurokitchen.quire.resources.reader_zoom_reset
import dev.neurokitchen.quire.resources.reading_read
import dev.neurokitchen.quire.resources.reading_reading
import dev.neurokitchen.quire.resources.reading_unread
import org.jetbrains.compose.resources.stringResource

/** PDF/Text switch, zoom (PDF), Highlight (Text, with a selection), and the reading status. */
@Composable
fun ReaderToolbar(
    mode: ReaderMode,
    hasPdf: Boolean,
    canHighlight: Boolean,
    reading: ReadingStatus,
    onModeChange: (ReaderMode) -> Unit,
    onZoomIn: () -> Unit,
    onZoomOut: () -> Unit,
    onZoomReset: () -> Unit,
    onHighlight: () -> Unit,
    onReadingChange: (ReadingStatus) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        modifier = modifier
            .fillMaxWidth()
            .height(Dimens.ToolbarHeight)
            .background(QuireTheme.colors.toolbar)
            .padding(horizontal = Dimens.SpaceXs),
    ) {
        Chip(stringResource(Res.string.reader_mode_pdf), selected = mode == ReaderMode.Pdf, enabled = hasPdf) {
            onModeChange(ReaderMode.Pdf)
        }
        Chip(stringResource(Res.string.reader_mode_text), selected = mode == ReaderMode.Text) { onModeChange(ReaderMode.Text) }
        Spacer(Modifier.width(Dimens.SpaceSm))
        when (mode) {
            ReaderMode.Pdf -> {
                val zoomOut = stringResource(Res.string.reader_zoom_out)
                val zoomIn = stringResource(Res.string.reader_zoom_in)
                TextButton(onClick = onZoomOut, modifier = Modifier.semantics { contentDescription = zoomOut }) {
                    Text("−", style = MaterialTheme.typography.titleLarge)
                }
                TextButton(onClick = onZoomReset) { Text(stringResource(Res.string.reader_zoom_reset)) }
                TextButton(onClick = onZoomIn, modifier = Modifier.semantics { contentDescription = zoomIn }) {
                    Text("+", style = MaterialTheme.typography.titleLarge)
                }
            }
            ReaderMode.Text -> TextButton(onClick = onHighlight, enabled = canHighlight) {
                Text(stringResource(Res.string.reader_highlight))
            }
        }
        Spacer(Modifier.weight(1f))
        listOf(
            ReadingStatus.Unread to Res.string.reading_unread,
            ReadingStatus.Reading to Res.string.reading_reading,
            ReadingStatus.Read to Res.string.reading_read,
        ).forEach { (status, label) -> Chip(stringResource(label), selected = reading == status) { onReadingChange(status) } }
    }
}

@Composable
private fun Chip(text: String, selected: Boolean, enabled: Boolean = true, onClick: () -> Unit) {
    val colors = QuireTheme.colors
    Text(
        text,
        style = MaterialTheme.typography.bodyMedium,
        color = if (selected) colors.onAccent else colors.fillPrimary,
        modifier = Modifier
            .alpha(if (enabled) 1f else 0.4f)
            .clip(RoundedCornerShape(Dimens.RadiusLarge))
            .background(if (selected) colors.accent else colors.fillQuinary)
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = Dimens.SpaceXs, vertical = 4.dp),
    )
}

