package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.presentation.components.cite.CiteButton
import dev.neurokitchen.quire.presentation.components.reference_card.DeleteReferenceConfirmationDialog
import dev.neurokitchen.quire.presentation.components.reference_card.LibraryReferenceRow
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceActionsSheet
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardEvent
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.mobile.navigation.ShelfKey
import dev.neurokitchen.quire.presentation.mobile.screens.home.HomeEvent
import dev.neurokitchen.quire.presentation.mobile.screens.home.HomeProjectSummary
import dev.neurokitchen.quire.presentation.mobile.screens.home.HomeViewModel
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.home_continue
import dev.neurokitchen.quire.resources.home_continue_sub
import dev.neurokitchen.quire.resources.home_days_count_plural
import dev.neurokitchen.quire.resources.home_days_read
import dev.neurokitchen.quire.resources.home_empty
import dev.neurokitchen.quire.resources.home_in_progress
import dev.neurokitchen.quire.resources.home_projects_sub
import dev.neurokitchen.quire.resources.home_projects_title
import dev.neurokitchen.quire.resources.home_queued
import dev.neurokitchen.quire.resources.home_recent
import dev.neurokitchen.quire.resources.home_recent_sub
import dev.neurokitchen.quire.resources.home_resume
import dev.neurokitchen.quire.resources.home_see_all
import dev.neurokitchen.quire.resources.home_start_reading
import dev.neurokitchen.quire.resources.home_status_completed
import dev.neurokitchen.quire.resources.home_streak_badge_plural
import dev.neurokitchen.quire.resources.home_up_next
import dev.neurokitchen.quire.resources.home_up_next_sub
import dev.neurokitchen.quire.presentation.screens.notifications.NotificationsViewModel
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_notifications
import dev.neurokitchen.quire.resources.library_unfiled
import dev.neurokitchen.quire.resources.mobile_items
import dev.neurokitchen.quire.resources.notifications
import dev.neurokitchen.quire.resources.reading_reading
import dev.neurokitchen.quire.resources.reading_unread
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** How many papers a Home section shows before "See all". */
private const val HOME_SECTION_SIZE = 5

/**
 * Home: what a phone opens to.
 * - Top bar with Quire brand mark, total paper count, and Notifications icon button
 * - Layered Hero Bento: Reading habit tracker (with today ring) + 3-tile Library Pulse (Reading / Unread / Completed)
 * - Horizontal Spotlight Carousel for Continue Reading (or Up Next queue)
 * - Quick-jump Projects & Shelves strip
 * - Recently Added list using unified LibraryReferenceRow cards
 */
@Composable
fun HomeTab(
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    onOpenShelf: (ShelfKey) -> Unit,
    onOpenNotifications: () -> Unit = {},
    viewModel: HomeViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val notifications by koinViewModel<NotificationsViewModel>().state.collectAsStateWithLifecycle()
    val colors = QuireTheme.colors
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

    Box(Modifier.fillMaxSize().background(colors.background)) {
        LazyColumn(
            Modifier.fillMaxSize().hazeSource(LocalMobileChrome.current.haze),
            contentPadding = mobileContentPadding(topBar = false),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
        ) {
            item {
                LargeTitle(
                    title = "Quire",
                    subtitle = pluralStringResource(Res.plurals.mobile_items, state.totalCount, state.totalCount),
                    icon = {
                        Box(
                            modifier = Modifier
                                .size(42.dp)
                                .clip(CircleShape)
                                .background(colors.cardBackground)
                                .clickable(
                                    onClickLabel = stringResource(Res.string.notifications),
                                    onClick = onOpenNotifications,
                                ),
                            contentAlignment = Alignment.Center,
                        ) {
                            Icon(
                                painter = painterResource(Res.drawable.ic_notifications),
                                contentDescription = stringResource(Res.string.notifications),
                                tint = colors.fillPrimary,
                                modifier = Modifier.size(21.dp),
                            )
                            if (notifications.unread > 0) {
                                Box(
                                    modifier = Modifier
                                        .align(Alignment.TopEnd)
                                        .padding(top = 5.dp, end = 5.dp)
                                        .clip(CircleShape)
                                        .background(colors.accent)
                                        .padding(horizontal = 5.dp, vertical = 1.dp),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    Text(
                                        text = if (notifications.unread > 99) "99+" else notifications.unread.toString(),
                                        style = MaterialTheme.typography.labelSmall.copy(
                                            fontSize = 10.sp,
                                            lineHeight = 12.sp,
                                            fontWeight = FontWeight.Bold,
                                        ),
                                        color = colors.onAccent,
                                    )
                                }
                            }
                        }
                    },
                )
            }
            item {
                HeroDashboardBento(
                    daysCount = state.daysCount,
                    streakCount = state.streakCount,
                    activeDaysOfWeek = state.activeDaysOfWeek,
                    todayIndex = state.todayIndex,
                    readingCount = state.readingCount,
                    unreadCount = state.unreadCount,
                    readCount = state.readCount,
                    onClickPulse = { onOpenShelf(all) },
                )
            }
            val error = state.error
            if (error != null) {
                item { EmptyNote(error) }
            } else if (!state.isLoading && state.reading.isEmpty() && state.recent.isEmpty()) {
                item { EmptyNote(stringResource(Res.string.home_empty)) }
            }
            continueReadingSection(
                cards = state.reading,
                isUpNext = state.isUpNextFallback,
                onOpenDetails = onOpenDetails,
                onOpenReader = onOpenReader,
                onLongClick = { viewModel.onEvent(HomeEvent.OnReferenceLongClick(it)) },
            )
            homeProjectsSection(
                projects = state.projects,
                unfiledCount = state.unfiledCount,
                onOpenShelf = onOpenShelf,
            )
            recentlyAddedSection(
                cards = state.recent,
                onOpenDetails = onOpenDetails,
                onOpenReader = onOpenReader,
                onLongClick = { viewModel.onEvent(HomeEvent.OnReferenceLongClick(it)) },
                onSeeAll = { onOpenShelf(all) },
            )
        }
    }
}

/**
 * Layered Hero Dashboard Bento combining:
 * 1) Reading habit & weekly momentum card (borderless surface with today indicator ring)
 * 2) 3-tile semantic Library Pulse strip (Reading · Unread · Completed)
 */
@Composable
fun HeroDashboardBento(
    daysCount: Int,
    streakCount: Int,
    activeDaysOfWeek: Set<Int> = setOf(0, 1, 2, 3, 4),
    todayIndex: Int = -1,
    readingCount: Int = 0,
    unreadCount: Int = 0,
    readCount: Int = 0,
    onClickPulse: () -> Unit = {},
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    val cardShape = RoundedCornerShape(20.dp)
    val tileShape = RoundedCornerShape(16.dp)

    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceXs),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        // Top Bento Card: Streak & Weekly Momentum (uniform 20.dp roundness, borderless)
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .clip(cardShape)
                .background(colors.cardBackground)
                .padding(16.dp),
        ) {
            Column {
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
                                .background(colors.accent.copy(alpha = 0.18f)),
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
                                text = stringResource(Res.string.home_days_read),
                                style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                                color = colors.fillSecondary,
                            )
                            Text(
                                text = pluralStringResource(Res.plurals.home_days_count_plural, daysCount, daysCount),
                                style = MaterialTheme.typography.titleMedium.copy(
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 19.sp,
                                ),
                                color = colors.fillPrimary,
                            )
                        }
                    }

                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(colors.accent.copy(alpha = 0.18f))
                            .padding(horizontal = 11.dp, vertical = 6.dp),
                    ) {
                        Text(
                            text = pluralStringResource(Res.plurals.home_streak_badge_plural, streakCount, streakCount),
                            style = MaterialTheme.typography.labelSmall.copy(
                                fontWeight = FontWeight.Bold,
                                fontSize = 12.sp,
                            ),
                            color = colors.accent,
                        )
                    }
                }

                Spacer(Modifier.height(16.dp))

                val days = listOf("M", "T", "W", "T", "F", "S", "S")
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    days.forEachIndexed { index, day ->
                        val isActive = index in activeDaysOfWeek
                        val isToday = index == todayIndex
                        Column(
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.spacedBy(5.dp),
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(34.dp)
                                    .clip(CircleShape)
                                    .background(if (isActive) colors.accent else colors.fillQuinary)
                                    .then(
                                        if (isToday && !isActive) {
                                            Modifier.border(1.5.dp, colors.accent.copy(alpha = 0.7f), CircleShape)
                                        } else {
                                            Modifier
                                        },
                                    ),
                                contentAlignment = Alignment.Center,
                            ) {
                                if (isActive) {
                                    Icon(
                                        painter = painterResource(Res.drawable.ic_description),
                                        contentDescription = null,
                                        tint = colors.onAccent,
                                        modifier = Modifier.size(16.dp),
                                    )
                                } else if (isToday) {
                                    Box(
                                        modifier = Modifier
                                            .size(6.dp)
                                            .clip(CircleShape)
                                            .background(colors.accent),
                                    )
                                }
                            }
                            Text(
                                text = day,
                                style = MaterialTheme.typography.labelSmall.copy(
                                    fontSize = 11.sp,
                                    fontWeight = if (isActive || isToday) FontWeight.Bold else FontWeight.Normal,
                                ),
                                color = if (isActive || isToday) colors.accent else colors.fillSecondary,
                            )
                        }
                    }
                }
            }
        }

        // Bottom Bento Row: 3-Tile Semantic Library Pulse (uniform 16.dp roundness)
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            PulseBentoTile(
                label = stringResource(Res.string.reading_reading),
                count = readingCount,
                tint = colors.statusReading,
                shape = tileShape,
                onClick = onClickPulse,
                modifier = Modifier.weight(1f),
            )
            PulseBentoTile(
                label = stringResource(Res.string.reading_unread),
                count = unreadCount,
                tint = colors.statusUnread,
                shape = tileShape,
                onClick = onClickPulse,
                modifier = Modifier.weight(1f),
            )
            PulseBentoTile(
                label = stringResource(Res.string.home_status_completed),
                count = readCount,
                tint = colors.statusRead,
                shape = tileShape,
                onClick = onClickPulse,
                modifier = Modifier.weight(1f),
            )
        }
    }
}

@Composable
private fun PulseBentoTile(
    label: String,
    count: Int,
    tint: Color,
    shape: RoundedCornerShape,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Column(
        modifier = modifier
            .clip(shape)
            .background(colors.cardBackground)
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 11.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Box(
                modifier = Modifier
                    .size(7.dp)
                    .clip(CircleShape)
                    .background(tint),
            )
            Text(
                text = count.toString(),
                style = MaterialTheme.typography.titleMedium.copy(
                    fontWeight = FontWeight.Bold,
                    fontSize = 17.sp,
                ),
                color = colors.fillPrimary,
            )
        }
        Text(
            text = label,
            style = MaterialTheme.typography.labelSmall.copy(
                fontSize = 11.sp,
                fontWeight = FontWeight.Medium,
            ),
            color = colors.fillSecondary,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

private fun LazyListScope.continueReadingSection(
    cards: List<ReferenceCardState>,
    isUpNext: Boolean,
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    onLongClick: (ReferenceCardState) -> Unit,
) {
    if (cards.isEmpty()) return
    val spotlight = cards.take(HOME_SECTION_SIZE)
    item(key = "header-continue-reading") {
        Column(
            Modifier.fillMaxWidth().padding(start = Dimens.SpaceLg, end = Dimens.SpaceLg, top = Dimens.SpaceMd, bottom = 4.dp),
        ) {
            Text(
                stringResource(if (isUpNext) Res.string.home_up_next else Res.string.home_continue),
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                color = QuireTheme.colors.fillPrimary,
            )
            Text(
                stringResource(if (isUpNext) Res.string.home_up_next_sub else Res.string.home_continue_sub),
                style = MaterialTheme.typography.bodySmall,
                color = QuireTheme.colors.fillSecondary,
            )
        }
    }
    if (spotlight.size == 1) {
        val card = spotlight.first()
        item(key = "continue-${card.id}") {
            Box(Modifier.padding(horizontal = Dimens.SpaceMd, vertical = 2.dp)) {
                ContinueReadingCard(
                    card = card,
                    isUpNext = isUpNext,
                    onOpenDetails = onOpenDetails,
                    onOpenReader = onOpenReader,
                    onLongClick = onLongClick,
                )
            }
        }
    } else {
        item(key = "continue-carousel") {
            LazyRow(
                contentPadding = PaddingValues(horizontal = Dimens.SpaceMd),
                horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
                modifier = Modifier.fillMaxWidth().padding(vertical = 2.dp),
            ) {
                items(spotlight, key = { "continue-${it.id}" }) { card ->
                    ContinueReadingCard(
                        card = card,
                        isUpNext = isUpNext,
                        onOpenDetails = onOpenDetails,
                        onOpenReader = onOpenReader,
                        onLongClick = onLongClick,
                        modifier = Modifier.width(308.dp),
                    )
                }
            }
        }
    }
}

private fun LazyListScope.homeProjectsSection(
    projects: List<HomeProjectSummary>,
    unfiledCount: Int,
    onOpenShelf: (ShelfKey) -> Unit,
) {
    if (projects.isEmpty() && unfiledCount == 0) return
    item(key = "header-projects") {
        Column(
            Modifier.fillMaxWidth().padding(start = Dimens.SpaceLg, end = Dimens.SpaceLg, top = Dimens.SpaceMd, bottom = 4.dp),
        ) {
            Text(
                stringResource(Res.string.home_projects_title),
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                color = QuireTheme.colors.fillPrimary,
            )
            Text(
                stringResource(Res.string.home_projects_sub),
                style = MaterialTheme.typography.bodySmall,
                color = QuireTheme.colors.fillSecondary,
            )
        }
    }
    item(key = "projects-carousel") {
        val colors = QuireTheme.colors
        LazyRow(
            contentPadding = PaddingValues(horizontal = Dimens.SpaceMd),
            horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
            modifier = Modifier.fillMaxWidth().padding(vertical = 2.dp),
        ) {
            items(projects, key = { "proj-${it.path}" }) { proj ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier
                        .height(40.dp)
                        .clip(RoundedCornerShape(14.dp))
                        .background(colors.cardBackground)
                        .clickable {
                            onOpenShelf(ShelfKey(ShelfKey.Kind.Project, path = proj.path, name = proj.name))
                        }
                        .padding(horizontal = 14.dp),
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_folder),
                        contentDescription = null,
                        tint = colors.sectionProjects,
                        modifier = Modifier.size(17.dp),
                    )
                    Text(
                        text = proj.name,
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.SemiBold,
                        color = colors.fillPrimary,
                    )
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(colors.sectionProjects.copy(alpha = 0.15f))
                            .padding(horizontal = 7.dp, vertical = 2.dp),
                    ) {
                        Text(
                            text = proj.count.toString(),
                            style = MaterialTheme.typography.labelSmall,
                            fontWeight = FontWeight.Bold,
                            color = colors.sectionProjects,
                        )
                    }
                }
            }
            if (unfiledCount > 0) {
                item(key = "proj-unfiled") {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        modifier = Modifier
                            .height(40.dp)
                            .clip(RoundedCornerShape(14.dp))
                            .background(colors.cardBackground)
                            .clickable { onOpenShelf(ShelfKey(ShelfKey.Kind.Unfiled)) }
                            .padding(horizontal = 14.dp),
                    ) {
                        Icon(
                            painter = painterResource(Res.drawable.ic_description),
                            contentDescription = null,
                            tint = colors.fillSecondary,
                            modifier = Modifier.size(16.dp),
                        )
                        Text(
                            text = stringResource(Res.string.library_unfiled),
                            style = MaterialTheme.typography.labelLarge,
                            fontWeight = FontWeight.Medium,
                            color = colors.fillPrimary,
                        )
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusFull))
                                .background(colors.fillQuinary)
                                .padding(horizontal = 7.dp, vertical = 2.dp),
                        ) {
                            Text(
                                text = unfiledCount.toString(),
                                style = MaterialTheme.typography.labelSmall,
                                fontWeight = FontWeight.Bold,
                                color = colors.fillSecondary,
                            )
                        }
                    }
                }
            }
        }
    }
}

private fun LazyListScope.recentlyAddedSection(
    cards: List<ReferenceCardState>,
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    onLongClick: (ReferenceCardState) -> Unit,
    onSeeAll: (() -> Unit)?,
) {
    if (cards.isEmpty()) return
    val recentSlice = cards.take(HOME_SECTION_SIZE)
    item(key = "header-recently-added") {
        Row(
            Modifier.fillMaxWidth().padding(start = Dimens.SpaceLg, end = Dimens.SpaceXs, top = Dimens.SpaceMd, bottom = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(
                    stringResource(Res.string.home_recent),
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = QuireTheme.colors.fillPrimary,
                )
                Text(
                    stringResource(Res.string.home_recent_sub),
                    style = MaterialTheme.typography.bodySmall,
                    color = QuireTheme.colors.fillSecondary,
                )
            }
            if (onSeeAll != null && cards.size > HOME_SECTION_SIZE) {
                TextButton(onClick = onSeeAll) { Text(stringResource(Res.string.home_see_all)) }
            }
        }
    }
    items(recentSlice, key = { "recent-${it.id}" }) { card ->
        Box(Modifier.padding(horizontal = Dimens.SpaceMd)) {
            LibraryReferenceRow(
                state = card,
                onEvent = { event ->
                    when (event) {
                        is ReferenceCardEvent.OnClick -> onOpenDetails(event.id)
                        is ReferenceCardEvent.OnOpen -> onOpenReader(event.id, card.title)
                        is ReferenceCardEvent.OnLongClick -> onLongClick(event.card)
                    }
                },
                action = { CiteButton(card.id, card.citeKey) },
            )
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun ContinueReadingCard(
    card: ReferenceCardState,
    isUpNext: Boolean = false,
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    onLongClick: (ReferenceCardState) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    val statusTone = if (isUpNext) colors.accent else colors.statusReading
    val primaryProject = card.projects.firstOrNull()

    Card(
        shape = RoundedCornerShape(20.dp),
        colors = CardDefaults.cardColors(
            containerColor = colors.cardBackground,
        ),
        border = null,
        elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
        modifier = modifier
            .fillMaxWidth()
            .height(188.dp)
            .clip(RoundedCornerShape(20.dp))
            .combinedClickable(
                onClick = { onOpenDetails(card.id) },
                onLongClick = { onLongClick(card) },
            ),
    ) {
        Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    // Status pill
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(5.dp),
                        modifier = Modifier
                            .clip(RoundedCornerShape(Dimens.RadiusFull))
                            .background(statusTone.copy(alpha = 0.16f))
                            .padding(horizontal = 8.dp, vertical = 3.dp),
                    ) {
                        Box(
                            modifier = Modifier
                                .size(6.dp)
                                .clip(CircleShape)
                                .background(statusTone),
                        )
                        Text(
                            text = stringResource(if (isUpNext) Res.string.home_queued else Res.string.home_in_progress),
                            style = MaterialTheme.typography.labelSmall.copy(
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Bold,
                                letterSpacing = 0.6.sp,
                            ),
                            color = statusTone,
                        )
                    }

                    // Project pill if assigned
                    if (!primaryProject.isNullOrBlank()) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(4.dp),
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusFull))
                                .background(colors.sectionProjects.copy(alpha = 0.14f))
                                .padding(horizontal = 8.dp, vertical = 3.dp),
                        ) {
                            Icon(
                                painter = painterResource(Res.drawable.ic_folder),
                                contentDescription = null,
                                tint = colors.sectionProjects,
                                modifier = Modifier.size(11.dp),
                            )
                            Text(
                                text = primaryProject,
                                style = MaterialTheme.typography.labelSmall.copy(
                                    fontSize = 10.sp,
                                    fontWeight = FontWeight.SemiBold,
                                ),
                                color = colors.sectionProjects,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }

                if (card.year.isNotBlank()) {
                    Text(
                        text = card.year,
                        style = MaterialTheme.typography.labelSmall.copy(
                            fontFamily = FontFamily.Monospace,
                            fontSize = 11.sp,
                        ),
                        color = colors.fillSecondary,
                    )
                }
            }

            Spacer(Modifier.height(10.dp))

            Text(
                text = card.title,
                style = MaterialTheme.typography.titleMedium.copy(
                    fontFamily = FontFamily.Serif,
                    fontWeight = FontWeight.SemiBold,
                    fontSize = 16.sp,
                    lineHeight = 22.sp,
                ),
                minLines = 2,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                color = colors.fillPrimary,
            )

            val metaLine = listOf(card.creator, card.venue).filter { it.isNotBlank() }.joinToString(" · ")
            Spacer(Modifier.height(4.dp))
            Text(
                text = metaLine.ifBlank { " " },
                style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                color = colors.fillSecondary,
            )

            Spacer(Modifier.weight(1f))

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Button(
                    onClick = { onOpenReader(card.id, card.title) },
                    shape = RoundedCornerShape(Dimens.RadiusFull),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = colors.accent,
                        contentColor = colors.onAccent,
                    ),
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 6.dp),
                    modifier = Modifier.height(36.dp),
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_library_books),
                        contentDescription = null,
                        modifier = Modifier.size(16.dp),
                    )
                    Spacer(Modifier.width(6.dp))
                    Text(
                        text = stringResource(if (isUpNext) Res.string.home_start_reading else Res.string.home_resume),
                        style = MaterialTheme.typography.labelMedium.copy(
                            fontWeight = FontWeight.Bold,
                            fontSize = 12.sp,
                        ),
                    )
                }

                CiteButton(card.id, card.citeKey)
            }
        }
    }
}
