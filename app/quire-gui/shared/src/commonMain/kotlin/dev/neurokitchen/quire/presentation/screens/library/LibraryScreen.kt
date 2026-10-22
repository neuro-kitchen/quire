package dev.neurokitchen.quire.presentation.screens.library

import dev.neurokitchen.quire.presentation.components.export_dialog.ExportTarget
import dev.neurokitchen.quire.presentation.components.export_dialog.ExportDialog
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
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCard
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardEvent
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.ExpressiveSprings
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.components.ActivityRail
import dev.neurokitchen.quire.presentation.screens.library.components.AddReferenceDialog
import dev.neurokitchen.quire.presentation.screens.library.components.ConfirmDialog
import dev.neurokitchen.quire.presentation.screens.library.components.FolderHierarchySheet
import dev.neurokitchen.quire.presentation.screens.library.components.ItemPane
import dev.neurokitchen.quire.presentation.screens.library.components.ItemsTable
import dev.neurokitchen.quire.presentation.screens.library.components.ItemsToolbar
import dev.neurokitchen.quire.presentation.screens.library.components.LibraryEmptyState
import dev.neurokitchen.quire.presentation.screens.library.components.LibraryErrorState
import dev.neurokitchen.quire.presentation.screens.library.components.LibrarySearchField
import dev.neurokitchen.quire.presentation.screens.library.components.NameDialogView
import dev.neurokitchen.quire.presentation.screens.library.components.ProjectsPane
import dev.neurokitchen.quire.presentation.screens.library.components.QuickLookModal
import dev.neurokitchen.quire.presentation.screens.library.components.ReferenceDetailPane
import dev.neurokitchen.quire.presentation.screens.library.components.ReferenceDrag
import dev.neurokitchen.quire.resources.ic_add
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_draft
import dev.neurokitchen.quire.resources.ic_expand_more
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_trash
import dev.neurokitchen.quire.resources.library_unfiled
import org.jetbrains.compose.resources.painterResource
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
                OnePane(state, onEvent, onOpenReference)
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

    state.addDialog?.let { dialog -> AddReferenceDialog(dialog, onEvent, projects = state.projectChoices) }
    state.exportDialog?.let { target ->
        ExportDialog(
            target = target,
            projects = state.projectChoices,
            onDismiss = { onEvent(LibraryEvent.OnShowExport(null)) },
            onDone = { onEvent(LibraryEvent.OnExportDone(it)) },
        )
    }

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
                    onExportSelected = { onEvent(LibraryEvent.OnShowExport(ExportTarget(references = state.selectedIds.toList()))) },
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

/** Phones and small windows: the list with collection selector & quick add, or the selected reference. */
@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
private fun OnePane(
    state: LibraryState,
    onEvent: (LibraryEvent) -> Unit,
    onOpenReference: (String, String) -> Unit,
) {
    val colors = QuireTheme.colors
    val selected = state.selected
    if (selected != null) {
        ReferenceDetailPane(
            reference = selected,
            onClose = { onEvent(LibraryEvent.OnCloseDetail) },
            onOpen = { onOpenReference(selected.id, selected.title) },
        )
        return
    }

    var isHierarchySheetOpen by remember { mutableStateOf(false) }

    val currentSourceName = when (val src = state.source) {
        LibrarySource.All -> stringResource(Res.string.library_my_library)
        LibrarySource.Unfiled -> stringResource(Res.string.library_unfiled)
        LibrarySource.Trash -> stringResource(Res.string.library_trash)
        is LibrarySource.InProject -> state.projects.find { it.id == src.path }?.name ?: src.path
    }

    val currentSourceIcon = when (state.source) {
        LibrarySource.All -> Res.drawable.ic_library_books
        LibrarySource.Unfiled -> Res.drawable.ic_draft
        LibrarySource.Trash -> Res.drawable.ic_delete
        is LibrarySource.InProject -> Res.drawable.ic_folder
    }

    Column(
        Modifier
            .fillMaxSize()
            .pointerInput(Unit) {
                detectHorizontalDragGestures { _, dragAmount ->
                    // Swipe left opens the full folder hierarchy sheet
                    if (dragAmount < -30f) {
                        isHierarchySheetOpen = true
                    }
                }
            }
            .padding(horizontal = 20.dp)
            .padding(top = 12.dp),
    ) {
        // Editorial Top Header
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp),
        ) {
            Column {
                Text(
                    text = buildAnnotatedString {
                        withStyle(
                            SpanStyle(
                                fontWeight = FontWeight.Normal,
                                color = colors.fillPrimary,
                            ),
                        ) {
                            append("Quire ")
                        }
                        withStyle(
                            SpanStyle(
                                fontWeight = FontWeight.ExtraBold,
                                color = colors.accent,
                            ),
                        ) {
                            append("Home")
                        }
                    },
                    style = MaterialTheme.typography.headlineMedium.copy(
                        fontSize = 26.sp,
                        letterSpacing = (-0.5).sp,
                    ),
                )
                Spacer(Modifier.height(2.dp))
                val itemCount = state.references.size
                Text(
                    text = "$currentSourceName • $itemCount ${if (itemCount == 1) "item" else "items"}",
                    style = MaterialTheme.typography.bodySmall.copy(fontSize = 13.sp),
                    color = colors.fillSecondary,
                )
            }
        }

        // Quick Collection Carousel / Filter Pills
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            modifier = Modifier.fillMaxWidth().padding(top = 4.dp, bottom = 8.dp),
        ) {
            item {
                val isSelected = state.source == LibrarySource.All
                CollectionFilterChip(
                    selected = isSelected,
                    label = stringResource(Res.string.library_my_library),
                    icon = Res.drawable.ic_library_books,
                    onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.All)) },
                )
            }
            item {
                val isSelected = state.source == LibrarySource.Unfiled
                CollectionFilterChip(
                    selected = isSelected,
                    label = stringResource(Res.string.library_unfiled),
                    icon = Res.drawable.ic_draft,
                    onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.Unfiled)) },
                )
            }
            items(state.projects) { project ->
                val isSelected = state.source == LibrarySource.InProject(project.id)
                CollectionFilterChip(
                    selected = isSelected,
                    label = project.name,
                    icon = Res.drawable.ic_folder,
                    onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject(project.id))) },
                )
            }
        }

        state.searchError?.let { SearchError(it) }
        val error = state.error
        if (error != null) {
            LibraryErrorState(error)
        } else if (!state.isLoading && state.references.isEmpty()) {
            LibraryEmptyState(state.query)
        } else {
            LazyColumn(
                contentPadding = PaddingValues(top = 4.dp, bottom = 24.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                items(state.references, key = { it.id }) { card ->
                    ReferenceCard(card, onEvent = { event ->
                        when (event) {
                            is ReferenceCardEvent.OnClick -> onEvent(LibraryEvent.OnReferenceClick(event.id))
                            is ReferenceCardEvent.OnOpen -> onOpenReference(event.id, card.title)
                            else -> Unit
                        }
                    })
                }
            }
        }
    }

    if (isHierarchySheetOpen) {
        FolderHierarchySheet(
            currentSource = state.source,
            projects = state.projects,
            onSelectSource = { onEvent(LibraryEvent.OnSourceClick(it)) },
            onDismiss = { isHierarchySheetOpen = false },
        )
    }
}

@Composable
private fun CollectionFilterChip(
    selected: Boolean,
    label: String,
    icon: org.jetbrains.compose.resources.DrawableResource,
    onClick: () -> Unit,
) {
    val colors = QuireTheme.colors
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .clip(RoundedCornerShape(Dimens.RadiusFull))
            .background(if (selected) colors.accent.copy(alpha = 0.16f) else colors.cardBackground)
            .clickable(onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 8.dp),
    ) {
        Icon(
            painter = painterResource(icon),
            contentDescription = null,
            tint = if (selected) colors.accent else colors.fillSecondary,
            modifier = Modifier.size(16.dp),
        )
        Spacer(Modifier.width(8.dp))
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium.copy(
                fontWeight = if (selected) FontWeight.Bold else FontWeight.Medium,
                fontSize = 13.sp,
            ),
            color = if (selected) colors.accent else colors.fillPrimary,
        )
    }
}
