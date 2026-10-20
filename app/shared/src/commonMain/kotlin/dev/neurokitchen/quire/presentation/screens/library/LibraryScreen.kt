package dev.neurokitchen.quire.presentation.screens.library

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCard
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardEvent
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.components.ItemPane
import dev.neurokitchen.quire.presentation.screens.library.components.ItemsTable
import dev.neurokitchen.quire.presentation.screens.library.components.ItemsToolbar
import dev.neurokitchen.quire.presentation.screens.library.components.LibraryEmptyState
import dev.neurokitchen.quire.presentation.screens.library.components.LibraryErrorState
import dev.neurokitchen.quire.presentation.screens.library.components.LibrarySearchField
import dev.neurokitchen.quire.presentation.screens.library.components.ProjectsPane
import dev.neurokitchen.quire.presentation.screens.library.components.ReferenceDetailPane
import org.koin.compose.viewmodel.koinViewModel

/** Zotero's three panes need about this much (its minimum pane widths add up to ~930). */
private val ThreePaneWidth = 960.dp

/** `onOpenReference` opens a paper in the reader (its ID and title). */
@Composable
fun LibraryRoute(
    onOpenReference: (String, String) -> Unit = { _, _ -> },
    viewModel: LibraryViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LibraryScreen(state, viewModel::onEvent, onOpenReference = onOpenReference)
}

@Composable
fun LibraryScreen(
    state: LibraryState,
    onEvent: (LibraryEvent) -> Unit,
    modifier: Modifier = Modifier,
    onOpenReference: (String, String) -> Unit = { _, _ -> },
) {
    BoxWithConstraints(modifier.fillMaxSize()) {
        if (maxWidth >= ThreePaneWidth) ThreePanes(state, onEvent, onOpenReference) else OnePane(state, onEvent)
    }
}

/** Desktop and large tablets: projects and tags | items | item pane, like Zotero. */
@Composable
private fun ThreePanes(state: LibraryState, onEvent: (LibraryEvent) -> Unit, onOpen: (String, String) -> Unit) {
    val divider = QuireTheme.colors.paneDivider
    Row(Modifier.fillMaxSize()) {
        ProjectsPane(state, onEvent, Modifier.width(Dimens.ProjectsPaneWidth))
        VerticalDivider(color = divider)
        Column(Modifier.weight(1f).fillMaxHeight()) {
            ItemsToolbar(state.query, onQueryChange = { onEvent(LibraryEvent.OnQueryChange(it)) })
            HorizontalDivider(color = divider)
            val error = state.error
            if (error != null) {
                LibraryErrorState(error)
            } else if (!state.isLoading && state.references.isEmpty()) {
                LibraryEmptyState(state.query)
            } else {
                ItemsTable(
                    references = state.references,
                    sort = state.sort,
                    ascending = state.ascending,
                    onSortClick = { onEvent(LibraryEvent.OnSortClick(it)) },
                    onReferenceClick = { onEvent(LibraryEvent.OnReferenceClick(it)) },
                    onReferenceOpen = { id -> state.references.firstOrNull { it.id == id }?.let { onOpen(id, it.title) } },
                )
            }
        }
        VerticalDivider(color = divider)
        ItemPane(
            reference = state.selected,
            itemCount = state.references.size,
            collapsed = state.collapsedSections,
            onSectionToggle = { onEvent(LibraryEvent.OnSectionToggle(it)) },
            onOpen = { state.selected?.let { onOpen(it.id, it.title) } },
            modifier = Modifier.width(Dimens.ItemPaneWidth),
        )
    }
}

/** Phones and small windows: the list, or the selected reference. */
@Composable
private fun OnePane(state: LibraryState, onEvent: (LibraryEvent) -> Unit) {
    val selected = state.selected
    if (selected != null) {
        ReferenceDetailPane(selected, onClose = { onEvent(LibraryEvent.OnCloseDetail) })
        return
    }
    Column(Modifier.fillMaxSize().padding(16.dp)) {
        LibrarySearchField(state.query, onQueryChange = { onEvent(LibraryEvent.OnQueryChange(it)) })
        val error = state.error
        if (error != null) {
            LibraryErrorState(error)
        } else if (!state.isLoading && state.references.isEmpty()) {
            LibraryEmptyState(state.query)
        } else {
            LazyColumn(
                contentPadding = PaddingValues(vertical = 12.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(state.references, key = { it.id }) { card ->
                    ReferenceCard(card, onEvent = { event ->
                        when (event) {
                            is ReferenceCardEvent.OnClick -> onEvent(LibraryEvent.OnReferenceClick(event.id))
                        }
                    })
                }
            }
        }
    }
}
