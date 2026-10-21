package dev.neurokitchen.quire.presentation.mobile.screens

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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.ReadingStatus
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
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.destination_library
import dev.neurokitchen.quire.resources.home_continue
import dev.neurokitchen.quire.resources.home_empty
import dev.neurokitchen.quire.resources.home_recent
import dev.neurokitchen.quire.resources.home_see_all
import dev.neurokitchen.quire.resources.home_unread
import dev.neurokitchen.quire.resources.ic_chevron_right
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_draft
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import dev.neurokitchen.quire.resources.ic_sell
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_trash
import dev.neurokitchen.quire.resources.library_unfiled
import dev.neurokitchen.quire.resources.mobile_items
import dev.neurokitchen.quire.resources.mobile_library_projects
import dev.neurokitchen.quire.resources.mobile_library_saved
import dev.neurokitchen.quire.resources.mobile_library_tags
import dev.neurokitchen.quire.resources.mobile_nothing_here
import dev.neurokitchen.quire.resources.mobile_toggle_children
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** How many papers a Home section shows before "See all". */
private const val HOME_SECTION_SIZE = 4

/**
 * Home: what a phone opens to. Papers being read first, then the newest, then unread ones; each
 * section leads to its full list.
 */
@Composable
fun HomeTab(
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    onOpenShelf: (ShelfKey) -> Unit,
    viewModel: LibraryViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val references = state.references
    val reading = references.filter { it.reading == ReadingStatus.Reading }
    val recent = references.sortedByDescending { it.dateAdded }
    val unread = references.filter { it.reading == ReadingStatus.Unread }
    val all = ShelfKey(ShelfKey.Kind.All)
    Box(Modifier.fillMaxSize()) {
        LazyColumn(
            Modifier.fillMaxSize().hazeSource(LocalMobileChrome.current.haze),
            contentPadding = mobileContentPadding(topBar = false),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
        ) {
            item { LargeTitle("Quire", stringResource(Res.string.library_my_library) + " · " + pluralStringResource(Res.plurals.mobile_items, references.size, references.size)) }
            if (!state.isLoading && references.isEmpty()) {
                item { EmptyNote(stringResource(Res.string.home_empty)) }
            }
            section(Res.string.home_continue, reading, onOpenDetails, onOpenReader, onSeeAll = null)
            section(Res.string.home_recent, recent, onOpenDetails, onOpenReader, onSeeAll = { onOpenShelf(all) })
            section(Res.string.home_unread, unread, onOpenDetails, onOpenReader, onSeeAll = { onOpenShelf(all) })
        }
    }
}

private fun LazyListScope.section(
    title: org.jetbrains.compose.resources.StringResource,
    cards: List<ReferenceCardState>,
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
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
            ReferenceCard(card, onEvent = { event ->
                when (event) {
                    is ReferenceCardEvent.OnClick -> onOpenDetails(event.id)
                    is ReferenceCardEvent.OnOpen -> onOpenReader(event.id, card.title)
                }
            })
        }
    }
}

/**
 * The Library tab: where papers are kept. The library, unfiled papers, projects (nested, opened
 * in place), tags, saved searches and the trash; each opens its list.
 */
@Composable
fun LibraryTab(
    onOpenShelf: (ShelfKey) -> Unit,
    viewModel: LibraryViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val total = state.references.size.takeIf { state.source == LibrarySource.All }
    LazyColumn(
        Modifier.fillMaxSize().hazeSource(LocalMobileChrome.current.haze),
        contentPadding = mobileContentPadding(topBar = false),
    ) {
        item { LargeTitle(stringResource(Res.string.destination_library), null) }
        item {
            Group {
                ShelfRow(Res.drawable.ic_library_books, stringResource(Res.string.library_my_library), total) { onOpenShelf(ShelfKey(ShelfKey.Kind.All)) }
                ShelfRow(Res.drawable.ic_draft, stringResource(Res.string.library_unfiled), null) { onOpenShelf(ShelfKey(ShelfKey.Kind.Unfiled)) }
            }
        }
        if (state.projects.isNotEmpty()) {
            item { GroupLabel(stringResource(Res.string.mobile_library_projects)) }
            item {
                Group {
                    state.projects.forEach { project ->
                        ShelfRow(
                            icon = Res.drawable.ic_folder,
                            label = project.name,
                            count = project.referenceCount,
                            depth = project.depth,
                            expanded = if (project.hasChildren) project.isExpanded else null,
                            onToggle = { viewModel.onEvent(LibraryEvent.OnProjectExpandToggle(project.id)) },
                        ) { onOpenShelf(ShelfKey(ShelfKey.Kind.Project, project.id, project.name)) }
                    }
                }
            }
        }
        if (state.tags.isNotEmpty()) {
            item { GroupLabel(stringResource(Res.string.mobile_library_tags)) }
            item {
                Group {
                    state.tags.forEach { tag ->
                        ShelfRow(Res.drawable.ic_sell, tag.name, null) { onOpenShelf(ShelfKey(ShelfKey.Kind.Tag, name = tag.name)) }
                    }
                }
            }
        }
        if (state.savedSearches.isNotEmpty()) {
            item { GroupLabel(stringResource(Res.string.mobile_library_saved)) }
            item {
                Group {
                    state.savedSearches.forEach { saved ->
                        ShelfRow(Res.drawable.ic_auto_fix_high, saved.name, null) { onOpenShelf(ShelfKey(ShelfKey.Kind.SavedSearch, name = saved.name)) }
                    }
                }
            }
        }
        item { Spacer(Modifier.size(Dimens.SpaceMd)) }
        item {
            Group {
                ShelfRow(Res.drawable.ic_delete, stringResource(Res.string.library_trash), null) { onOpenShelf(ShelfKey(ShelfKey.Kind.Trash)) }
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
                ReferenceCard(card, onEvent = { event ->
                    when (event) {
                        is ReferenceCardEvent.OnClick -> onOpenDetails(event.id)
                        is ReferenceCardEvent.OnOpen -> onOpenReader(event.id, card.title)
                    }
                })
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
