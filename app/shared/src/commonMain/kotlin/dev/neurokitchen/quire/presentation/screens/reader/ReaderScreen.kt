package dev.neurokitchen.quire.presentation.screens.reader

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.reader.components.AnnotationsSidebar
import dev.neurokitchen.quire.presentation.screens.reader.components.NotesPanel
import dev.neurokitchen.quire.presentation.screens.reader.components.PdfPagesView
import dev.neurokitchen.quire.presentation.screens.reader.components.ReaderToolbar
import dev.neurokitchen.quire.presentation.screens.reader.components.TextReaderView
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.reader_loading
import dev.neurokitchen.quire.resources.reader_no_pdf
import dev.neurokitchen.quire.resources.reader_no_text
import dev.neurokitchen.quire.resources.reader_source
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel
import org.koin.core.parameter.parametersOf

/** One paper in a reader tab; `id` is its reference ID. */
@Composable
fun ReaderRoute(
    id: String,
    title: String,
    viewModel: ReaderViewModel = koinViewModel(key = "reader-$id") { parametersOf(id, title) },
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    ReaderScreen(state, viewModel::onEvent, loadPage = viewModel::page, loadFigure = viewModel::figure)
}

/**
 * Zotero's reader layout: annotations on the left, the PDF pages or the text in the middle, notes
 * on the right. The text selection is local to the screen until it becomes a highlight.
 */
@Composable
fun ReaderScreen(
    state: ReaderState,
    onEvent: (ReaderEvent) -> Unit,
    loadPage: suspend (Int, Int) -> PageImage?,
    loadFigure: suspend (String) -> String?,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    var selection by remember(state.referenceId) { mutableStateOf<Pair<String, String>?>(null) }
    val located = state.pdf.locations.map { it.highlightId }.toSet()
    Row(modifier.fillMaxSize()) {
        AnnotationsSidebar(
            highlights = state.highlights,
            focused = state.focusedHighlight,
            missing = if (state.mode == ReaderMode.Pdf) state.highlights.map { it.id }.filterNot { it in located }.toSet() else emptySet(),
            onClick = { onEvent(ReaderEvent.OnHighlightClick(it)) },
            onComment = { id, text -> onEvent(ReaderEvent.OnHighlightComment(id, text)) },
            onRemove = { onEvent(ReaderEvent.OnHighlightRemove(it)) },
            modifier = Modifier.width(SidebarWidth),
        )
        VerticalDivider(color = colors.paneDivider)
        Column(Modifier.weight(1f).fillMaxHeight()) {
            ReaderToolbar(
                mode = state.mode,
                hasPdf = state.pdf.path != null,
                canHighlight = selection != null,
                reading = state.reading,
                onModeChange = { onEvent(ReaderEvent.OnModeChange(it)) },
                onZoomIn = { onEvent(ReaderEvent.OnZoomIn) },
                onZoomOut = { onEvent(ReaderEvent.OnZoomOut) },
                onZoomReset = { onEvent(ReaderEvent.OnZoomReset) },
                onHighlight = {
                    selection?.let { (quote, prefix) -> onEvent(ReaderEvent.OnHighlight(quote, prefix)) }
                    selection = null
                },
                onReadingChange = { onEvent(ReaderEvent.OnReadingChange(it)) },
            )
            HorizontalDivider(color = colors.paneDivider)
            StatusLine(state)
            when {
                state.mode == ReaderMode.Pdf && state.pdf.pages.isNotEmpty() -> PdfPagesView(
                    pages = state.pdf.pages,
                    zoom = state.pdf.zoom,
                    locations = state.pdf.locations,
                    focused = state.focusedHighlight,
                    loadPage = loadPage,
                )
                state.isLoading && state.tabs.isEmpty() -> Message(stringResource(Res.string.reader_loading))
                else -> TextReaderView(
                    tabs = state.tabs,
                    selectedTab = state.selectedTab,
                    highlights = state.highlights,
                    focused = state.focusedHighlight,
                    onTabClick = { onEvent(ReaderEvent.OnTabClick(it)) },
                    onSelection = { selection = it },
                    loadFigure = loadFigure,
                )
            }
        }
        VerticalDivider(color = colors.paneDivider)
        NotesPanel(
            notes = state.notes,
            draft = state.noteDraft,
            onDraftChange = { onEvent(ReaderEvent.OnNoteDraftChange(it)) },
            onAdd = { onEvent(ReaderEvent.OnNoteAdd) },
            onRemove = { onEvent(ReaderEvent.OnNoteRemove(it)) },
            modifier = Modifier.width(NotesWidth),
        )
    }
}

private val SidebarWidth = 260.dp
private val NotesWidth = 280.dp

/** Where the text came from, why there is none, or what failed. */
@Composable
private fun StatusLine(state: ReaderState) {
    val colors = QuireTheme.colors
    val (text, isError) = when {
        state.error != null -> state.error to true
        state.mode == ReaderMode.Pdf && state.pdf.error != null -> state.pdf.error to true
        state.mode == ReaderMode.Text && state.textError != null -> stringResource(Res.string.reader_no_text, state.textError) to false
        state.mode == ReaderMode.Text && state.source != null -> stringResource(Res.string.reader_source, state.source) to false
        state.mode == ReaderMode.Pdf && state.pdf.path == null && !state.isLoading -> stringResource(Res.string.reader_no_pdf) to false
        else -> return
    }
    Text(
        text,
        style = MaterialTheme.typography.bodySmall,
        color = if (isError) colors.accentRed else colors.fillSecondary,
        modifier = Modifier.padding(horizontal = Dimens.SpaceMd, vertical = 4.dp),
    )
}

@Composable
private fun Message(text: String) {
    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Text(text, style = MaterialTheme.typography.bodyLarge, color = QuireTheme.colors.fillSecondary)
    }
}
