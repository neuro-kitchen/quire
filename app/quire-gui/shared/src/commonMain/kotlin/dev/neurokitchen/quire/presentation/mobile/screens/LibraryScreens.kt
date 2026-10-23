package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.presentation.components.cite.CiteButton
import dev.neurokitchen.quire.presentation.components.notice.NoticeBar
import dev.neurokitchen.quire.presentation.components.reference_card.DeleteReferenceConfirmationDialog
import dev.neurokitchen.quire.presentation.components.reference_card.LibraryReferenceRow
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceActionsSheet
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCard
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardEvent
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.mobile.navigation.ShelfKey
import dev.neurokitchen.quire.presentation.mobile.screens.library.MobileLibraryEvent
import dev.neurokitchen.quire.presentation.mobile.screens.library.MobileLibraryViewModel
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryState
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.library.components.FolderHierarchySheet
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.destination_library
import dev.neurokitchen.quire.resources.ic_quire_logo
import dev.neurokitchen.quire.resources.ic_sync
import dev.neurokitchen.quire.resources.ic_tune
import dev.neurokitchen.quire.resources.item_restore
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_new_collection
import dev.neurokitchen.quire.resources.library_trash
import dev.neurokitchen.quire.resources.library_unfiled
import dev.neurokitchen.quire.resources.mobile_items
import dev.neurokitchen.quire.resources.mobile_nothing_here
import dev.neurokitchen.quire.resources.name_cancel
import dev.neurokitchen.quire.resources.name_create
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/**
 * The Library tab: displays the full list of papers in a stacked layout,
 * with the folder carousel and bottom sheet tree view at the top.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LibraryTab(
    onOpenShelf: (ShelfKey) -> Unit = {},
    onOpenDetails: (String) -> Unit = {},
    onOpenReader: (String, String) -> Unit = { _, _ -> },
    viewModel: MobileLibraryViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val clipboardManager = LocalClipboardManager.current

    state.actionSheetReference?.let { card ->
        ReferenceActionsSheet(
            reference = card,
            onDismiss = { viewModel.onEvent(MobileLibraryEvent.OnDismissActionSheet) },
            onCopyBib = {
                clipboardManager.setText(AnnotatedString(it.toBibtex()))
            },
            onCopyDoi = {
                if (it.doi.isNotBlank()) {
                    val doiText = if (it.doi.startsWith("10.")) "https://doi.org/${it.doi}" else it.doi
                    clipboardManager.setText(AnnotatedString(doiText))
                }
            },
            onDownloadPdf = {
                viewModel.onEvent(MobileLibraryEvent.OnDownloadPdf(it.id))
            },
            onDeleteRequest = {
                viewModel.onEvent(MobileLibraryEvent.OnRequestDelete(it))
            },
        )
    }

    state.confirmDeleteReference?.let { card ->
        DeleteReferenceConfirmationDialog(
            reference = card,
            onConfirm = { viewModel.onEvent(MobileLibraryEvent.OnConfirmDelete) },
            onDismiss = { viewModel.onEvent(MobileLibraryEvent.OnDismissDeleteDialog) },
        )
    }

    if (state.isFolderSheetVisible) {
        FolderHierarchySheet(
            currentSource = state.source,
            projects = state.projects,
            onSelectSource = { source ->
                viewModel.onEvent(MobileLibraryEvent.OnSelectSource(source))
            },
            onDismiss = {
                viewModel.onEvent(MobileLibraryEvent.SetFolderSheetVisible(false))
            },
            onAddProject = {
                viewModel.onEvent(MobileLibraryEvent.OnOpenCreateProjectDialog)
            },
        )
    }

    if (state.isCreateProjectDialogOpen) {
        var name by remember { mutableStateOf("") }
        AlertDialog(
            onDismissRequest = { viewModel.onEvent(MobileLibraryEvent.OnDismissCreateProjectDialog) },
            title = { Text(stringResource(Res.string.library_new_collection)) },
            text = {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(stringResource(Res.string.library_new_collection)) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
            },
            confirmButton = {
                TextButton(
                    onClick = { viewModel.onEvent(MobileLibraryEvent.OnCreateProject(name)) },
                    enabled = name.isNotBlank(),
                ) {
                    Text(stringResource(Res.string.name_create))
                }
            },
            dismissButton = {
                TextButton(onClick = { viewModel.onEvent(MobileLibraryEvent.OnDismissCreateProjectDialog) }) {
                    Text(stringResource(Res.string.name_cancel))
                }
            },
        )
    }

    Box(Modifier.fillMaxSize().background(QuireTheme.colors.background)) {
        PullToRefreshBox(
            isRefreshing = state.isRefreshing,
            onRefresh = { viewModel.onEvent(MobileLibraryEvent.OnRefresh) },
            modifier = Modifier.fillMaxSize(),
        ) {
            LazyColumn(
                Modifier.fillMaxSize().hazeSource(LocalMobileChrome.current.haze),
                contentPadding = mobileContentPadding(topBar = false),
                verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
            ) {
                item {
                    LargeTitle(
                        stringResource(Res.string.destination_library),
                        pluralStringResource(Res.plurals.mobile_items, state.references.size, state.references.size),
                    )
                }
                item {
                    LibraryFilterRow(
                        source = state.source,
                        projects = state.projectChoices,
                        onOpenFolders = { viewModel.onEvent(MobileLibraryEvent.SetFolderSheetVisible(true)) },
                        onSelectSource = { viewModel.onEvent(MobileLibraryEvent.OnSelectSource(it)) },
                    )
                }
                item {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(start = Dimens.SpaceLg, end = Dimens.SpaceLg, top = 2.dp, bottom = 4.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = pluralStringResource(Res.plurals.mobile_items, state.references.size, state.references.size),
                            style = MaterialTheme.typography.labelSmall.copy(fontSize = 12.sp, fontWeight = FontWeight.Medium),
                            color = QuireTheme.colors.fillSecondary,
                        )
                    }
                }
                val error = state.error
                if (error != null) {
                    item { EmptyNote(error) }
                } else if (!state.isLoading && state.references.isEmpty()) {
                    item { EmptyNote(stringResource(Res.string.mobile_nothing_here)) }
                }
                items(state.references, key = { it.id }) { card ->
                    Box(Modifier.padding(horizontal = Dimens.SpaceMd)) {
                        LibraryReferenceRow(
                            card,
                            onEvent = { event ->
                                when (event) {
                                    is ReferenceCardEvent.OnClick -> onOpenDetails(event.id)
                                    is ReferenceCardEvent.OnOpen -> onOpenReader(event.id, card.title)
                                    is ReferenceCardEvent.OnLongClick -> viewModel.onEvent(MobileLibraryEvent.OnReferenceLongClick(event.card))
                                }
                            },
                            action = { CiteButton(card.id, card.citeKey) },
                        )
                    }
                }
            }
        }
    }
}

/** A list of papers (the library, a project, a tag…), opened from the Library or Home tab. */
@Composable
fun ShelfScreen(
    shelf: ShelfKey,
    onBack: () -> Unit,
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    viewModel: LibraryViewModel = koinViewModel(),
) {
    LaunchedEffect(shelf) {
        viewModel.onEvent(LibraryEvent.OnSourceClick(shelf.toSource()))
        when (shelf.kind) {
            ShelfKey.Kind.Tag -> viewModel.onEvent(LibraryEvent.OnTagClick(shelf.name))
            ShelfKey.Kind.SavedSearch -> viewModel.onEvent(LibraryEvent.OnSavedSearchClick(shelf.name))
            else -> {}
        }
    }
    val state by viewModel.state.collectAsStateWithLifecycle()
    val title = shelf.title()
    val chrome = LocalMobileChrome.current
    val isTrash = shelf.kind == ShelfKey.Kind.Trash

    Box(Modifier.fillMaxSize().background(QuireTheme.colors.background)) {
        ReferenceList(
            state = state,
            onOpenDetails = onOpenDetails,
            onOpenReader = onOpenReader,
            onRestore = if (isTrash) { id -> viewModel.onEvent(LibraryEvent.OnRestoreId(id)) } else null,
            modifier = Modifier.hazeSource(chrome.haze),
        )
        MobileTopBar(title, onBack, chrome.haze)
        state.notice?.let { notice ->
            NoticeBar(
                notice = notice,
                onDismiss = { viewModel.onEvent(LibraryEvent.OnDismissNotice) },
                modifier = Modifier
                    .align(Alignment.BottomCenter)
                    .padding(bottom = chrome.bottomBar + Dimens.SpaceMd, start = Dimens.SpaceMd, end = Dimens.SpaceMd),
            )
        }
    }
}

@Composable
internal fun ReferenceList(
    state: LibraryState,
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    modifier: Modifier = Modifier,
    onRestore: ((String) -> Unit)? = null,
    header: (LazyListScope.() -> Unit)? = null,
    contentPadding: PaddingValues = mobileContentPadding(),
) {
    LazyColumn(
        modifier.fillMaxSize(),
        contentPadding = contentPadding,
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
    ) {
        header?.invoke(this)
        if (!state.isLoading && state.references.isEmpty()) {
            item { EmptyNote(stringResource(Res.string.mobile_nothing_here)) }
        }
        items(state.references, key = { it.id }) { card ->
            Box(Modifier.padding(horizontal = Dimens.SpaceMd)) {
                ReferenceCard(
                    card,
                    onEvent = { event ->
                        when (event) {
                            is ReferenceCardEvent.OnClick -> onOpenDetails(event.id)
                            is ReferenceCardEvent.OnOpen -> onOpenReader(event.id, card.title)
                            else -> Unit
                        }
                    },
                    showCiteKey = false,
                    action = if (onRestore != null) {
                        {
                            FilledTonalButton(
                                onClick = { onRestore(card.id) },
                                shape = RoundedCornerShape(Dimens.RadiusFull),
                                colors = ButtonDefaults.filledTonalButtonColors(
                                    containerColor = QuireTheme.colors.accent.copy(alpha = 0.16f),
                                    contentColor = QuireTheme.colors.accent,
                                ),
                                contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp),
                                modifier = Modifier.height(32.dp),
                            ) {
                                Icon(
                                    painter = painterResource(Res.drawable.ic_sync),
                                    contentDescription = stringResource(Res.string.item_restore),
                                    modifier = Modifier.size(14.dp),
                                )
                                Spacer(Modifier.width(6.dp))
                                Text(
                                    text = stringResource(Res.string.item_restore),
                                    style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Bold, fontSize = 12.sp),
                                )
                            }
                        }
                    } else {
                        { CiteButton(card.id, card.citeKey) }
                    },
                )
            }
        }
    }
}

/** A tab's large title (Material's large top app bar, scrolling with the content). */
@Composable
internal fun LargeTitle(
    title: String,
    subtitle: String?,
    icon: (@Composable () -> Unit)? = null,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(start = Dimens.SpaceLg, end = Dimens.SpaceLg, top = Dimens.SpaceLg, bottom = Dimens.SpaceXs),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Column(modifier = Modifier.weight(1f, fill = false)) {
            if (title == "Quire") {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Image(
                        painter = painterResource(Res.drawable.ic_quire_logo),
                        contentDescription = "Quire",
                        modifier = Modifier.size(34.dp).padding(end = 1.dp),
                    )
                    Text(
                        "uire",
                        style = MaterialTheme.typography.headlineLarge,
                        fontWeight = FontWeight.Bold,
                        color = QuireTheme.colors.fillPrimary,
                    )
                }
            } else {
                Text(title, style = MaterialTheme.typography.headlineLarge, fontWeight = FontWeight.Bold, color = QuireTheme.colors.fillPrimary)
            }
            if (subtitle != null) {
                Text(subtitle, style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary)
            }
        }
        if (icon != null) {
            Box(modifier = Modifier.padding(start = Dimens.SpaceSm)) {
                icon()
            }
        }
    }
}

@Composable
internal fun EmptyNote(text: String) {
    Text(
        text,
        style = MaterialTheme.typography.bodyLarge,
        color = QuireTheme.colors.fillSecondary,
        modifier = Modifier.fillMaxWidth().padding(Dimens.SpaceLg),
    )
}

@Composable
internal fun GroupLabel(text: String) {
    Text(
        text,
        style = MaterialTheme.typography.labelLarge,
        color = QuireTheme.colors.fillSecondary,
        modifier = Modifier.padding(start = Dimens.SpaceLg, end = Dimens.SpaceLg, top = Dimens.SpaceLg, bottom = Dimens.SpaceXs),
    )
}

/** Rows grouped on one rounded surface (Android's settings-style list). */
@Composable
internal fun Group(content: @Composable () -> Unit) {
    Column(
        Modifier
            .padding(horizontal = Dimens.SpaceMd)
            .fillMaxWidth()
            .clip(RoundedCornerShape(Dimens.RadiusLarge))
            .background(QuireTheme.colors.cardBackground),
    ) { content() }
}

@Composable
private fun ShelfKey.title(): String = when (kind) {
    ShelfKey.Kind.All -> stringResource(Res.string.library_my_library)
    ShelfKey.Kind.Unfiled -> stringResource(Res.string.library_unfiled)
    ShelfKey.Kind.Trash -> stringResource(Res.string.library_trash)
    ShelfKey.Kind.Project -> name.ifEmpty { path.orEmpty() }
    ShelfKey.Kind.Tag -> "#$name"
    ShelfKey.Kind.SavedSearch -> name
}

/**
 * Library filter carousel with the leading tune/settings button inspired by
 * Pocket Hoot budgeting app. Tapping the tune button opens the bottom sheet with
 * the folder tree structure.
 */
@Composable
internal fun LibraryFilterRow(
    source: LibrarySource,
    projects: List<Pair<String, String>>,
    onOpenFolders: () -> Unit,
    onSelectSource: (LibrarySource) -> Unit,
) {
    val colors = QuireTheme.colors
    val chips = listOf(
        Pair<LibrarySource, String>(LibrarySource.All, stringResource(Res.string.library_my_library)),
        Pair(LibrarySource.Unfiled, stringResource(Res.string.library_unfiled)),
    ) + projects.map { (path, name) -> Pair(LibrarySource.InProject(path), name) }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceXs),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier = Modifier
                .height(36.dp)
                .width(46.dp)
                .clip(RoundedCornerShape(Dimens.RadiusFull))
                .background(colors.fillQuinary)
                .clickable(onClick = onOpenFolders),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                painter = painterResource(Res.drawable.ic_tune),
                contentDescription = "Folders tree",
                tint = colors.fillPrimary,
                modifier = Modifier.size(18.dp),
            )
        }

        Spacer(Modifier.width(Dimens.SpaceXs))

        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            items(chips, key = { it.first.toString() }) { (chip, label) ->
                val isSelected = chip == source
                Box(
                    modifier = Modifier
                        .height(36.dp)
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .background(if (isSelected) colors.accent else colors.fillQuinary)
                        .clickable { onSelectSource(chip) }
                        .padding(horizontal = 16.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = label,
                        style = MaterialTheme.typography.labelMedium.copy(
                            fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                            fontSize = 13.sp,
                        ),
                        color = if (isSelected) colors.onAccent else colors.fillPrimary,
                    )
                }
            }
        }
    }
}
