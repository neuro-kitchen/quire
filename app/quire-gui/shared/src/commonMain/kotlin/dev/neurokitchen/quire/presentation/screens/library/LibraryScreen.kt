package dev.neurokitchen.quire.presentation.screens.library

import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCard
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardEvent
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.components.ActivityRail
import dev.neurokitchen.quire.presentation.screens.library.components.AddReferenceDialog
import dev.neurokitchen.quire.presentation.screens.library.components.CreateProjectDialog
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
    val searchFocusRequester = remember { FocusRequester() }
    val rootFocusRequester = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        rootFocusRequester.requestFocus()
    }

    Box(
        modifier = modifier
            .fillMaxSize()
            .focusRequester(rootFocusRequester)
            .focusable()
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown) {
                    if (event.isCtrlPressed) {
                        when (event.key) {
                            Key.B -> {
                                onEvent(LibraryEvent.OnToggleSidebar)
                                true
                            }
                            Key.I -> {
                                onEvent(LibraryEvent.OnToggleInspector)
                                true
                            }
                            Key.F, Key.S -> {
                                searchFocusRequester.requestFocus()
                                true
                            }
                            Key.N -> {
                                onEvent(LibraryEvent.OnShowAddDialog(true))
                                true
                            }
                            Key.A -> {
                                onEvent(LibraryEvent.OnSelectAll)
                                true
                            }
                            else -> false
                        }
                    } else {
                        when (event.key) {
                            Key.Delete, Key.Backspace -> {
                                onEvent(LibraryEvent.OnDeleteSelected)
                                true
                            }
                            Key.Escape -> {
                                onEvent(LibraryEvent.OnClearSelection)
                                true
                            }
                            else -> false
                        }
                    }
                } else {
                    false
                }
            },
    ) {
        BoxWithConstraints(Modifier.fillMaxSize()) {
            if (maxWidth >= ThreePaneWidth) {
                ThreePanes(state, onEvent, onOpenReference, searchFocusRequester)
            } else {
                OnePane(state, onEvent)
            }
        }
    }

    if (state.showAddDialog) {
        AddReferenceDialog(
            onDismiss = { onEvent(LibraryEvent.OnShowAddDialog(false)) },
            onAdd = { onEvent(LibraryEvent.OnAddReference(it)) },
        )
    }

    if (state.showAddProjectDialog) {
        CreateProjectDialog(
            onDismiss = { onEvent(LibraryEvent.OnShowAddProjectDialog(false)) },
            onCreate = { onEvent(LibraryEvent.OnCreateProject(it)) },
        )
    }
}

/** Desktop and large tablets: projects and tags | items | item pane, with resizable panes and minimizable sidebars. */
@Composable
private fun ThreePanes(
    state: LibraryState,
    onEvent: (LibraryEvent) -> Unit,
    onOpen: (String, String) -> Unit,
    searchFocusRequester: FocusRequester,
) {
    var leftPaneWidthDp by remember { androidx.compose.runtime.mutableFloatStateOf(260f) }
    var rightPaneWidthDp by remember { androidx.compose.runtime.mutableFloatStateOf(340f) }

    Row(Modifier.fillMaxSize()) {
        if (state.isSidebarCollapsed) {
            ActivityRail(state, onEvent)
        } else {
            ProjectsPane(state, onEvent, Modifier.width(leftPaneWidthDp.dp))
            dev.neurokitchen.quire.presentation.designsystem.components.DraggableSplitter(
                onDelta = { delta ->
                    leftPaneWidthDp = (leftPaneWidthDp + delta).coerceIn(180f, 450f)
                },
            )
        }
        Column(Modifier.weight(1f).fillMaxHeight()) {
            ItemsToolbar(
                query = state.query,
                onQueryChange = { onEvent(LibraryEvent.OnQueryChange(it)) },
                onAddReference = { onEvent(LibraryEvent.OnShowAddDialog(true)) },
                searchFocusRequester = searchFocusRequester,
                onToggleInspector = { onEvent(LibraryEvent.OnToggleInspector) },
                isInspectorCollapsed = state.isInspectorCollapsed,
            )
            HorizontalDivider(color = QuireTheme.colors.paneDivider)
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
                    selectedIds = state.selectedIds,
                    onSortClick = { onEvent(LibraryEvent.OnSortClick(it)) },
                    onReferenceClick = { onEvent(LibraryEvent.OnReferenceClick(it)) },
                    onReferenceSelect = { id, isCtrl, isShift -> onEvent(LibraryEvent.OnReferenceSelect(id, isCtrl, isShift)) },
                    onReferenceOpen = { id -> state.references.firstOrNull { it.id == id }?.let { onOpen(id, it.title) } },
                    onDeleteReference = { onEvent(LibraryEvent.OnDeleteReference(it)) },
                    onDeleteSelected = { onEvent(LibraryEvent.OnDeleteSelected) },
                    onSelectAll = { onEvent(LibraryEvent.OnSelectAll) },
                    onClearSelection = { onEvent(LibraryEvent.OnClearSelection) },
                    onCreateCollectionFromSelected = { onEvent(LibraryEvent.OnShowAddProjectDialog(true)) },
                )
            }
        }
        if (!state.isInspectorCollapsed) {
            dev.neurokitchen.quire.presentation.designsystem.components.DraggableSplitter(
                onDelta = { delta ->
                    rightPaneWidthDp = (rightPaneWidthDp - delta).coerceIn(240f, 600f)
                },
            )
            ItemPane(
                reference = state.selected,
                itemCount = state.references.size,
                collapsed = state.collapsedSections,
                onSectionToggle = { onEvent(LibraryEvent.OnSectionToggle(it)) },
                onOpen = { state.selected?.let { onOpen(it.id, it.title) } },
                onDelete = { state.selected?.let { onEvent(LibraryEvent.OnDeleteReference(it.id)) } },
                onAddReference = { onEvent(LibraryEvent.OnShowAddDialog(true)) },
                onCreateProject = { onEvent(LibraryEvent.OnShowAddProjectDialog(true)) },
                onCollapse = { onEvent(LibraryEvent.OnToggleInspector) },
                modifier = Modifier.width(rightPaneWidthDp.dp),
            )
        }
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
