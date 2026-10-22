package dev.neurokitchen.quire.presentation.mobile.screens

import dev.neurokitchen.quire.presentation.mobile.screens.home.HomeViewModel
import dev.neurokitchen.quire.presentation.mobile.screens.library.MobileLibraryViewModel
import dev.neurokitchen.quire.presentation.mobile.screens.library.MobileLibraryEvent
import dev.neurokitchen.quire.presentation.components.cite.CiteButton
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.foundation.lazy.LazyRow
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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.components.reference_card.DeleteReferenceConfirmationDialog
import dev.neurokitchen.quire.presentation.components.reference_card.LibraryReferenceRow
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceActionsSheet
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCard
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardEvent
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.mobile.navigation.ShelfKey
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryState
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.library.components.FolderHierarchySheet
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.destination_library
import dev.neurokitchen.quire.resources.home_continue
import dev.neurokitchen.quire.resources.home_empty
import dev.neurokitchen.quire.resources.home_recent
import dev.neurokitchen.quire.resources.home_see_all
import dev.neurokitchen.quire.resources.home_unread
import dev.neurokitchen.quire.resources.ic_chevron_right
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_draft
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import dev.neurokitchen.quire.resources.ic_sell
import dev.neurokitchen.quire.resources.ic_tune
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_trash
import dev.neurokitchen.quire.resources.library_unfiled
import dev.neurokitchen.quire.resources.mobile_items
import dev.neurokitchen.quire.resources.mobile_library_projects
import dev.neurokitchen.quire.resources.mobile_library_saved
import dev.neurokitchen.quire.resources.mobile_library_tags
import dev.neurokitchen.quire.resources.mobile_nothing_here
import dev.neurokitchen.quire.resources.mobile_toggle_children
import dev.neurokitchen.quire.presentation.mobile.screens.home.HomeEvent
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** How many papers a Home section shows before "See all". */
private const val HOME_SECTION_SIZE = 4

/**
 * Home: what a phone opens to.
 * Shows days read streak card, papers currently being read, and recently added papers.
 */
@Composable
fun HomeTab(
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    onOpenShelf: (ShelfKey) -> Unit,
    viewModel: HomeViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val all = ShelfKey(ShelfKey.Kind.All)
    val clipboardManager = LocalClipboardManager.current

    state.actionSheetReference?.let { card ->
        ReferenceActionsSheet(
            reference = card,
            onDismiss = { viewModel.onEvent(HomeEvent.OnDismissActionSheet) },
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
                viewModel.onEvent(HomeEvent.OnDownloadPdf(it.id))
            },
            onDeleteRequest = {
                viewModel.onEvent(HomeEvent.OnRequestDelete(it))
            },
        )
    }

    state.confirmDeleteReference?.let { card ->
        DeleteReferenceConfirmationDialog(
            reference = card,
            onConfirm = { viewModel.onEvent(HomeEvent.OnConfirmDelete) },
            onDismiss = { viewModel.onEvent(HomeEvent.OnDismissDeleteDialog) },
        )
    }

    Box(Modifier.fillMaxSize()) {
        LazyColumn(
            Modifier.fillMaxSize().hazeSource(LocalMobileChrome.current.haze),
            contentPadding = mobileContentPadding(topBar = false),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
        ) {
            item { LargeTitle("Quire", pluralStringResource(Res.plurals.mobile_items, state.totalCount, state.totalCount)) }
            item {
                DaysReadCard(
                    daysCount = state.daysCount,
                    streakCount = state.streakCount,
                    activeDaysOfWeek = state.activeDaysOfWeek,
                )
            }
            val error = state.error
            if (error != null) {
                // The engine could not read the library: say so (it is not an empty library).
                item { EmptyNote(error) }
            } else if (!state.isLoading && state.reading.isEmpty() && state.recent.isEmpty()) {
                item { EmptyNote(stringResource(Res.string.home_empty)) }
            }
            section(
                Res.string.home_continue,
                state.reading,
                onOpenDetails,
                onOpenReader,
                onLongClick = { viewModel.onEvent(HomeEvent.OnReferenceLongClick(it)) },
                onSeeAll = null,
            )
            section(
                Res.string.home_recent,
                state.recent,
                onOpenDetails,
                onOpenReader,
                onLongClick = { viewModel.onEvent(HomeEvent.OnReferenceLongClick(it)) },
                onSeeAll = { onOpenShelf(all) },
            )
        }
    }
}

/**
 * A habit/activity card displaying days read and weekly momentum,
 * inspired by Pocket Hoot & modern financial/habit apps.
 */
@Composable
fun DaysReadCard(
    daysCount: Int,
    streakCount: Int,
    activeDaysOfWeek: Set<Int> = setOf(0, 1, 2, 3, 4),
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Card(
        shape = RoundedCornerShape(20.dp),
        colors = CardDefaults.cardColors(containerColor = colors.cardBackground),
        border = null,
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceXs),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Box(
                        modifier = Modifier
                            .size(44.dp)
                            .clip(RoundedCornerShape(14.dp))
                            .background(colors.accent.copy(alpha = 0.15f)),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            painter = painterResource(Res.drawable.ic_library_books),
                            contentDescription = null,
                            tint = colors.accent,
                            modifier = Modifier.size(22.dp),
                        )
                    }
                    Column {
                        Text(
                            text = "Days Read",
                            style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                            color = colors.fillSecondary,
                        )
                        Text(
                            text = "$daysCount days",
                            style = MaterialTheme.typography.titleMedium.copy(
                                fontWeight = FontWeight.Bold,
                                fontSize = 18.sp,
                            ),
                            color = colors.fillPrimary,
                        )
                    }
                }

                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .background(colors.accent.copy(alpha = 0.15f))
                        .padding(horizontal = 10.dp, vertical = 5.dp),
                ) {
                    Text(
                        text = "🔥 $streakCount day streak",
                        style = MaterialTheme.typography.labelSmall.copy(
                            fontWeight = FontWeight.SemiBold,
                            fontSize = 12.sp,
                        ),
                        color = colors.accent,
                    )
                }
            }

            Spacer(Modifier.height(14.dp))

            val days = listOf("M", "T", "W", "T", "F", "S", "S")
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                days.forEachIndexed { index, day ->
                    val isActive = index in activeDaysOfWeek
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        Box(
                            modifier = Modifier
                                .size(32.dp)
                                .clip(CircleShape)
                                .background(if (isActive) colors.accent else colors.fillQuinary),
                            contentAlignment = Alignment.Center,
                        ) {
                            if (isActive) {
                                Icon(
                                    painter = painterResource(Res.drawable.ic_description),
                                    contentDescription = null,
                                    tint = colors.onAccent,
                                    modifier = Modifier.size(16.dp),
                                )
                            }
                        }
                        Text(
                            text = day,
                            style = MaterialTheme.typography.labelSmall.copy(
                                fontSize = 11.sp,
                                fontWeight = if (isActive) FontWeight.Bold else FontWeight.Normal,
                            ),
                            color = if (isActive) colors.accent else colors.fillSecondary,
                        )
                    }
                }
            }
        }
    }
}

private fun LazyListScope.section(
    title: org.jetbrains.compose.resources.StringResource,
    cards: List<ReferenceCardState>,
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    onLongClick: (ReferenceCardState) -> Unit,
    onSeeAll: (() -> Unit)?,
) {
    if (cards.isEmpty()) return
    item(key = "header-$title") {
        Row(
            Modifier.fillMaxWidth().padding(start = Dimens.SpaceLg, end = Dimens.SpaceXs, top = Dimens.SpaceMd),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(stringResource(title), style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = QuireTheme.colors.fillPrimary, modifier = Modifier.weight(1f))
            if (onSeeAll != null && cards.size > HOME_SECTION_SIZE) {
                androidx.compose.material3.TextButton(onClick = onSeeAll) { Text(stringResource(Res.string.home_see_all)) }
            }
        }
    }
    items(cards.take(HOME_SECTION_SIZE), key = { "$title-${it.id}" }) { card ->
        Box(Modifier.padding(horizontal = Dimens.SpaceMd)) {
            ReferenceCard(
                card,
                onEvent = { event ->
                    when (event) {
                        is ReferenceCardEvent.OnClick -> onOpenDetails(event.id)
                        is ReferenceCardEvent.OnOpen -> onOpenReader(event.id, card.title)
                        is ReferenceCardEvent.OnLongClick -> onLongClick(event.card)
                    }
                },
                showCiteKey = false,
                action = { CiteButton(card.id, card.citeKey) },
            )
        }
    }
}

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
        )
    }

    Box(Modifier.fillMaxSize()) {
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
    Box(Modifier.fillMaxSize()) {
        ReferenceList(state, onOpenDetails, onOpenReader, Modifier.hazeSource(chrome.haze))
        MobileTopBar(title, onBack, chrome.haze)
    }
}

@Composable
internal fun ReferenceList(
    state: LibraryState,
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    modifier: Modifier = Modifier,
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
                    action = { CiteButton(card.id, card.citeKey) },
                )
            }
        }
    }
}

/** A tab's large title (Material's large top app bar, scrolling with the content). */
@Composable
internal fun LargeTitle(title: String, subtitle: String?) {
    Column(Modifier.fillMaxWidth().padding(start = Dimens.SpaceLg, end = Dimens.SpaceLg, top = Dimens.SpaceLg, bottom = Dimens.SpaceXs)) {
        Text(title, style = MaterialTheme.typography.headlineLarge, fontWeight = FontWeight.Bold, color = QuireTheme.colors.fillPrimary)
        if (subtitle != null) {
            Text(subtitle, style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary)
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

/** One place in the library: icon, name, count; a project with subprojects has a toggle. */
@Composable
private fun ShelfRow(
    icon: DrawableResource,
    label: String,
    count: Int?,
    depth: Int = 0,
    expanded: Boolean? = null,
    onToggle: () -> Unit = {},
    onClick: () -> Unit,
) {
    val colors = QuireTheme.colors
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = 56.dp)
            .clickable(onClick = onClick)
            .padding(start = Dimens.SpaceMd + Dimens.SpaceMd * depth, end = Dimens.SpaceXs),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
    ) {
        Icon(painterResource(icon), contentDescription = null, tint = colors.accent, modifier = Modifier.size(22.dp))
        Text(label, style = MaterialTheme.typography.bodyLarge, color = colors.fillPrimary, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
        if (count != null) {
            Text(count.toString(), style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary)
        }
        if (expanded != null) {
            IconButton(onClick = onToggle) {
                Icon(
                    painterResource(Res.drawable.ic_chevron_right),
                    contentDescription = stringResource(Res.string.mobile_toggle_children),
                    tint = colors.fillSecondary,
                    modifier = Modifier.rotate(if (expanded) 90f else 0f),
                )
            }
        } else {
            Spacer(Modifier.width(Dimens.SpaceXs))
        }
    }
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
        // Tune/filter button matching Pocket Hoot budgeting app reference
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
