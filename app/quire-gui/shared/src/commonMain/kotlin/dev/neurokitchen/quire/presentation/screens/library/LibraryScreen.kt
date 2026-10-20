package dev.neurokitchen.quire.presentation.screens.library

import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.presentation.components.notice.NoticeBar
import dev.neurokitchen.quire.resources.search_invalid
import org.jetbrains.compose.resources.stringResource
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.ui.layout.LayoutCoordinates
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.foundation.layout.offset
import androidx.compose.ui.unit.IntOffset
import kotlin.math.roundToInt
import org.jetbrains.compose.resources.pluralStringResource
import dev.neurokitchen.quire.resources.drag_references
import dev.neurokitchen.quire.resources.Res
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.ui.draw.clip
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.background
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
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
import dev.neurokitchen.quire.presentation.screens.library.components.ConfirmDialog
import dev.neurokitchen.quire.presentation.screens.library.components.NameDialogView
import dev.neurokitchen.quire.presentation.screens.library.components.ReferenceDrag
import dev.neurokitchen.quire.presentation.screens.library.components.ItemPane
import dev.neurokitchen.quire.presentation.screens.library.components.ItemsTable
import dev.neurokitchen.quire.presentation.screens.library.components.ItemsToolbar
import dev.neurokitchen.quire.presentation.screens.library.components.LibraryEmptyState
import dev.neurokitchen.quire.presentation.screens.library.components.LibraryErrorState
import dev.neurokitchen.quire.presentation.screens.library.components.LibrarySearchField
import dev.neurokitchen.quire.presentation.screens.library.components.ProjectsPane
import dev.neurokitchen.quire.presentation.screens.library.components.QuickLookModal
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
    val drag = remember { ReferenceDrag() }
    val rootCoordinates = remember { arrayOfNulls<LayoutCoordinates>(1) }

    LaunchedEffect(Unit) {
        rootFocusRequester.requestFocus()
    }

    Box(
        modifier = modifier
            .fillMaxSize()
            .onGloballyPositioned { rootCoordinates[0] = it }
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
                            Key.Spacebar -> {
                                if (state.selected != null) {
                                    onEvent(LibraryEvent.OnToggleQuickLook())
                                    true
                                } else false
                            }
                            Key.Delete, Key.Backspace -> {
                                onEvent(LibraryEvent.OnDeleteSelected)
                                true
                            }
                            Key.Escape -> {
                                if (state.isQuickLookOpen) {
                                    onEvent(LibraryEvent.OnToggleQuickLook(false))
                                    true
                                } else {
                                    onEvent(LibraryEvent.OnClearSelection)
                                    true
                                }
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
                ThreePanes(state, onEvent, onOpenReference, searchFocusRequester, drag)
            } else {
                OnePane(state, onEvent)
            }
        }
        if (drag.ids.isNotEmpty()) {
            val at = rootCoordinates[0]?.takeIf { it.isAttached }?.windowToLocal(drag.position) ?: drag.position
            DragBadge(drag.ids.size, Modifier.offset { IntOffset(at.x.roundToInt() + 12, at.y.roundToInt() + 12) })
        }
        state.notice?.let { notice ->
            NoticeBar(
                notice = notice,
                onDismiss = { onEvent(LibraryEvent.OnDismissNotice) },
                modifier = Modifier.align(Alignment.BottomCenter).padding(Dimens.SpaceMd),
            )
        }
    }

    state.addDialog?.let { dialog -> AddReferenceDialog(dialog, onEvent) }

    val uriHandler = LocalUriHandler.current
    LaunchedEffect(state.pageToOpen) {
        state.pageToOpen?.let { page ->
            runCatching { uriHandler.openUri(page) }
            onEvent(LibraryEvent.OnPageOpened)
        }
    }

    state.nameDialog?.let { dialog ->
        NameDialogView(
            dialog = dialog,
            onDismiss = { onEvent(LibraryEvent.OnNameDialog(null)) },
            onConfirm = { onEvent(LibraryEvent.OnNameConfirm(it)) },
        )
    }
    state.confirm?.let { confirm ->
        ConfirmDialog(
            confirm = confirm,
            onDismiss = { onEvent(LibraryEvent.OnConfirm(null)) },
            onConfirm = { onEvent(LibraryEvent.OnConfirmed) },
        )
    }

    if (state.isQuickLookOpen && state.selected != null) {
        QuickLookModal(
            reference = state.selected,
            onDismissRequest = { onEvent(LibraryEvent.OnToggleQuickLook(false)) },
            onOpenReader = {
                onEvent(LibraryEvent.OnToggleQuickLook(false))
                onOpenReference(state.selected.id, state.selected.title)
            },
        )
    }
}

/** Why the engine refused the search as typed; the results shown are the last good ones. */
@Composable
private fun SearchError(message: String) {
    Text(
        stringResource(Res.string.search_invalid, message),
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.error,
        modifier = Modifier.fillMaxWidth().padding(horizontal = Dimens.SpaceSm, vertical = Dimens.SpaceMin),
    )
}

/** Follows the pointer while references are dragged: how many. */
@Composable
private fun DragBadge(count: Int, modifier: Modifier) {
    Text(
        pluralStringResource(Res.plurals.drag_references, count, count),
        style = MaterialTheme.typography.labelMedium,
        color = QuireTheme.colors.onAccent,
        modifier = modifier
            .clip(RoundedCornerShape(Dimens.RadiusFull))
            .background(QuireTheme.colors.accent)
            .padding(horizontal = Dimens.SpaceXs, vertical = 2.dp),
    )
}

/** Desktop and large tablets: projects and tags | items | item pane, with resizable panes and minimizable sidebars. */
@Composable
private fun ThreePanes(
    state: LibraryState,
    onEvent: (LibraryEvent) -> Unit,
    onOpen: (String, String) -> Unit,
    searchFocusRequester: FocusRequester,
    drag: ReferenceDrag,
) {
    var leftPaneWidthDp by remember { androidx.compose.runtime.mutableFloatStateOf(260f) }
    var rightPaneWidthDp by remember { androidx.compose.runtime.mutableFloatStateOf(340f) }

    Row(Modifier.fillMaxSize()) {
        if (state.isSidebarCollapsed) {
            ActivityRail(state, onEvent)
        } else {
            ProjectsPane(state, onEvent, Modifier.width(leftPaneWidthDp.dp), drag)
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
                onSaveSearch = { onEvent(LibraryEvent.OnNameDialog(NameDialog.SaveSearch(state.query))) },
                onImportPdf = { onEvent(LibraryEvent.OnImportPdf(it)) },
            )
            HorizontalDivider(color = QuireTheme.colors.paneDivider)
            state.searchError?.let { SearchError(it) }
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
                    onCreateCollectionFromSelected = { onEvent(LibraryEvent.OnNameDialog(NameDialog.NewProject(withIds = state.selectedIds))) },
                    onQuickLook = { onEvent(LibraryEvent.OnToggleQuickLook(true)) },
                    drag = drag,
                    onDropOnProject = { ids, path -> onEvent(LibraryEvent.OnBatchAddToProject(ids, path)) },
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
                onCreateProject = { onEvent(LibraryEvent.OnNameDialog(NameDialog.NewProject())) },
                onCollapse = { onEvent(LibraryEvent.OnToggleInspector) },
                modifier = Modifier.width(rightPaneWidthDp.dp),
                editing = state.editing,
                tagDraft = state.tagDraft,
                noteDraft = state.noteDraft,
                onEvent = onEvent,
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
        state.searchError?.let { SearchError(it) }
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
