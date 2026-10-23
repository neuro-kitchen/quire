package dev.neurokitchen.quire.presentation.screens.search

import dev.neurokitchen.quire.presentation.components.cite.CiteButton
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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCard
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardEvent
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.library.components.LibrarySearchField
import dev.neurokitchen.quire.presentation.screens.library.components.ReferenceDetailPane
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.destination_search
import dev.neurokitchen.quire.resources.ic_search
import org.jetbrains.compose.resources.stringResource
import org.jetbrains.compose.resources.painterResource
import org.koin.compose.viewmodel.koinViewModel

private enum class SearchFilter(val label: String) {
    All("All"),
    HasPdf("PDF"),
    Unread("Unread"),
    Reading("Reading"),
    Read("Read"),
}

@Composable
fun SearchRoute(
    onOpenReference: (String, String) -> Unit = { _, _ -> },
    viewModel: LibraryViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val selected = state.selected
    if (selected != null) {
        ReferenceDetailPane(
            reference = selected,
            onClose = { viewModel.onEvent(LibraryEvent.OnCloseDetail) },
            onOpen = { onOpenReference(selected.id, selected.title) },
        )
    } else {
        SearchScreen(
            query = state.query,
            onQueryChange = { viewModel.onEvent(LibraryEvent.OnQueryChange(it)) },
            references = state.references,
            onReferenceClick = { viewModel.onEvent(LibraryEvent.OnReferenceClick(it)) },
            onOpenReference = onOpenReference,
        )
    }
}

/**
 * Dedicated ergonomic Search screen:
 * - Search input easily accessible
 * - Quick filter chips for PDF status and reading progress
 * - Instant live query results
 */
@Composable
fun SearchScreen(
    query: String,
    onQueryChange: (String) -> Unit,
    references: List<dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState>,
    onReferenceClick: (String) -> Unit,
    onOpenReference: (String, String) -> Unit,
    modifier: Modifier = Modifier,
    /** Space the results leave at the bottom (the navigation bar over them). */
    bottomPadding: androidx.compose.ui.unit.Dp = 0.dp,
) {
    val colors = QuireTheme.colors
    var activeFilter by remember { mutableStateOf(SearchFilter.All) }

    val filtered = remember(references, activeFilter) {
        when (activeFilter) {
            SearchFilter.All -> references
            SearchFilter.HasPdf -> references.filter { it.hasPdf }
            SearchFilter.Unread -> references.filter { it.reading == ReadingStatus.Unread }
            SearchFilter.Reading -> references.filter { it.reading == ReadingStatus.Reading }
            SearchFilter.Read -> references.filter { it.reading == ReadingStatus.Read }
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(horizontal = 20.dp)
            .padding(top = 12.dp),
    ) {
        // Standardized Header matching Home, Library, and Settings
        Column(
            modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp),
        ) {
            Text(
                text = stringResource(Res.string.destination_search),
                style = MaterialTheme.typography.headlineLarge,
                fontWeight = FontWeight.Bold,
                color = colors.fillPrimary,
            )
            Spacer(Modifier.height(2.dp))
            Text(
                text = "Find papers by title, author, year, or identifier",
                style = MaterialTheme.typography.bodyMedium,
                color = colors.fillSecondary,
            )
        }

        // Ergonomic Search Bar
        LibrarySearchField(
            query = query,
            onQueryChange = onQueryChange,
            modifier = Modifier.fillMaxWidth(),
        )

        // Quick Filter Chips
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            modifier = Modifier.fillMaxWidth().padding(top = 10.dp, bottom = 8.dp),
        ) {
            items(SearchFilter.entries) { filter ->
                val isSelected = filter == activeFilter
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(Dimens.RadiusFull))
                        .background(if (isSelected) colors.accent.copy(alpha = 0.16f) else colors.cardBackground)
                        .clickable { activeFilter = filter }
                        .padding(horizontal = 14.dp, vertical = 8.dp),
                ) {
                    Text(
                        text = filter.label,
                        style = MaterialTheme.typography.labelMedium.copy(
                            fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                            fontSize = 13.sp,
                        ),
                        color = if (isSelected) colors.accent else colors.fillPrimary,
                    )
                }
            }
        }

        // Search Results List or Empty State
        if (query.isEmpty() && references.isEmpty()) {
            Box(
                modifier = Modifier.fillMaxSize().padding(top = 40.dp),
                contentAlignment = Alignment.TopCenter,
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_search),
                        contentDescription = null,
                        tint = colors.fillTertiary,
                        modifier = Modifier.size(48.dp),
                    )
                    Spacer(Modifier.height(12.dp))
                    Text(
                        text = "Type to search your references",
                        style = MaterialTheme.typography.bodyMedium.copy(fontSize = 15.sp),
                        color = colors.fillSecondary,
                    )
                }
            }
        } else if (filtered.isEmpty()) {
            Box(
                modifier = Modifier.fillMaxSize().padding(top = 40.dp),
                contentAlignment = Alignment.TopCenter,
            ) {
                Text(
                    text = "No references found",
                    style = MaterialTheme.typography.bodyMedium.copy(fontSize = 15.sp),
                    color = colors.fillSecondary,
                )
            }
        } else {
            Text(
                text = "${filtered.size} ${if (filtered.size == 1) "result" else "results"}",
                style = MaterialTheme.typography.labelSmall.copy(fontSize = 12.sp),
                color = colors.fillSecondary,
                modifier = Modifier.padding(vertical = 4.dp),
            )
            LazyColumn(
                contentPadding = PaddingValues(top = 4.dp, bottom = 24.dp + bottomPadding),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                items(filtered, key = { it.id }) { card ->
                    ReferenceCard(
                        card,
                        onEvent = { event ->
                            when (event) {
                                is ReferenceCardEvent.OnClick -> onReferenceClick(event.id)
                                is ReferenceCardEvent.OnOpen -> onOpenReference(event.id, card.title)
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
}
