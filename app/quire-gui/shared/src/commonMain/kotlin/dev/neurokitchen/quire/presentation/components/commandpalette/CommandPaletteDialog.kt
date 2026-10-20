package dev.neurokitchen.quire.presentation.components.commandpalette

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.desktop.OpenPaper
import dev.neurokitchen.quire.presentation.desktop.ToolsTabId
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_inventory_2
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.ic_settings
import dev.neurokitchen.quire.resources.ic_sync
import org.jetbrains.compose.resources.painterResource

enum class PaletteCategory(val title: String) {
    All("All"),
    Papers("Papers"),
    Actions("Actions"),
    Themes("Themes"),
    Tabs("Tabs"),
}

sealed interface PaletteItem {
    val title: String
    val subtitle: String?
    val category: PaletteCategory

    data class Action(
        override val title: String,
        override val subtitle: String?,
        val shortcut: String? = null,
        val iconType: ActionIcon = ActionIcon.Default,
        val onExecute: () -> Unit,
    ) : PaletteItem {
        override val category: PaletteCategory = PaletteCategory.Actions
    }

    data class ThemeAction(
        val theme: AppTheme,
        val isCurrent: Boolean,
        val onSelect: () -> Unit,
    ) : PaletteItem {
        override val title: String = "Switch Theme to ${theme.title}"
        override val subtitle: String? = if (isCurrent) "Active Theme" else "Set interface color scheme"
        override val category: PaletteCategory = PaletteCategory.Themes
    }

    data class TabAction(
        val paper: OpenPaper,
        val onSelect: () -> Unit,
    ) : PaletteItem {
        override val title: String = paper.title
        override val subtitle: String = "Open Tab"
        override val category: PaletteCategory = PaletteCategory.Tabs
    }

    data class PaperItem(
        val reference: Reference,
        val onSelect: () -> Unit,
    ) : PaletteItem {
        override val title: String = reference.title
        override val subtitle: String = buildString {
            if (reference.citeKey.isNotBlank()) append("[${reference.citeKey}] ")
            val author = reference.creators.firstOrNull()?.sortName
            if (author != null) append(author)
            if (reference.creators.size > 1) append(" et al.")
            if (reference.year != null) append(" (${reference.year})")
        }
        override val category: PaletteCategory = PaletteCategory.Papers
    }
}

enum class ActionIcon {
    Default, Settings, Sync, Tools, Library, Add
}

@Composable
fun CommandPaletteDialog(
    isOpen: Boolean,
    onDismissRequest: () -> Unit,
    papers: List<OpenPaper>,
    allReferences: List<Reference>,
    currentTheme: AppTheme,
    onOpenPaper: (id: String, title: String) -> Unit,
    onSelectTab: (id: String?) -> Unit,
    onSetTheme: (AppTheme) -> Unit,
    onOpenSettings: () -> Unit,
    onReload: () -> Unit,
    onOpenTools: () -> Unit,
    onOpenCitationGraph: () -> Unit = {},
) {
    if (!isOpen) return

    val colors = QuireTheme.colors
    var query by remember { mutableStateOf("") }
    var selectedCategory by remember { mutableStateOf(PaletteCategory.All) }
    var selectedIndex by remember { mutableIntStateOf(0) }
    val focusRequester = remember { FocusRequester() }
    val listState = rememberLazyListState()

    // Build the items registry
    val allItems = remember(query, selectedCategory, papers, allReferences, currentTheme) {
        val items = mutableListOf<PaletteItem>()

        // 1. Navigation / Global Actions
        items.add(
            PaletteItem.Action(
                title = "Go to My Library",
                subtitle = "Main library collection view",
                shortcut = "Esc",
                iconType = ActionIcon.Library,
                onExecute = { onSelectTab(null); onDismissRequest() },
            )
        )
        items.add(
            PaletteItem.Action(
                title = "Open Preferences & Settings",
                subtitle = "Application and reader configurations",
                shortcut = "Ctrl+,",
                iconType = ActionIcon.Settings,
                onExecute = { onOpenSettings(); onDismissRequest() },
            )
        )
        items.add(
            PaletteItem.Action(
                title = "Reload & Sync Library",
                subtitle = "Reopen library files and refresh engine state",
                shortcut = "Ctrl+R",
                iconType = ActionIcon.Sync,
                onExecute = { onReload(); onDismissRequest() },
            )
        )
        items.add(
            PaletteItem.Action(
                title = "Open Tools & Maintenance",
                subtitle = "Duplicate detection, sync checks, import/export",
                shortcut = null,
                iconType = ActionIcon.Tools,
                onExecute = { onOpenTools(); onDismissRequest() },
            )
        )
        items.add(
            PaletteItem.Action(
                title = "View Citation Network Graph",
                subtitle = "Interactive 2D graph of connections between library papers",
                shortcut = null,
                iconType = ActionIcon.Default,
                onExecute = { onOpenCitationGraph(); onDismissRequest() },
            )
        )

        // 2. Open Tabs
        papers.forEach { paper ->
            items.add(
                PaletteItem.TabAction(
                    paper = paper,
                    onSelect = { onSelectTab(paper.id); onDismissRequest() },
                )
            )
        }

        // 3. Themes
        AppTheme.entries.forEach { theme ->
            items.add(
                PaletteItem.ThemeAction(
                    theme = theme,
                    isCurrent = theme == currentTheme,
                    onSelect = { onSetTheme(theme); onDismissRequest() },
                )
            )
        }

        // 4. Papers in Library
        allReferences.forEach { ref ->
            items.add(
                PaletteItem.PaperItem(
                    reference = ref,
                    onSelect = { onOpenPaper(ref.id, ref.title); onDismissRequest() },
                )
            )
        }

        // Filter by category and query
        items.filter { item ->
            val matchesCategory = selectedCategory == PaletteCategory.All || item.category == selectedCategory
            val q = query.trim().lowercase()
            val matchesQuery = if (q.isEmpty()) {
                true
            } else {
                item.title.lowercase().contains(q) || (item.subtitle?.lowercase()?.contains(q) == true)
            }
            matchesCategory && matchesQuery
        }
    }

    // Keep selectedIndex within bounds
    LaunchedEffect(allItems.size) {
        if (selectedIndex >= allItems.size) {
            selectedIndex = (allItems.size - 1).coerceAtLeast(0)
        }
    }

    // Scroll into view when selectedIndex changes
    LaunchedEffect(selectedIndex) {
        if (selectedIndex in allItems.indices) {
            listState.animateScrollToItem(selectedIndex)
        }
    }

    Dialog(onDismissRequest = onDismissRequest) {
        Surface(
            shape = RoundedCornerShape(14.dp),
            color = colors.sidepane,
            shadowElevation = 12.dp,
            modifier = Modifier
                .width(620.dp)
                .border(1.dp, colors.border.copy(alpha = 0.8f), RoundedCornerShape(14.dp))
                .onKeyEvent { keyEvent ->
                    if (keyEvent.type == KeyEventType.KeyDown) {
                        when (keyEvent.key) {
                            Key.DirectionDown -> {
                                if (allItems.isNotEmpty()) {
                                    selectedIndex = (selectedIndex + 1) % allItems.size
                                }
                                true
                            }
                            Key.DirectionUp -> {
                                if (allItems.isNotEmpty()) {
                                    selectedIndex = if (selectedIndex <= 0) allItems.size - 1 else selectedIndex - 1
                                }
                                true
                            }
                            Key.Enter -> {
                                if (selectedIndex in allItems.indices) {
                                    when (val item = allItems[selectedIndex]) {
                                        is PaletteItem.Action -> item.onExecute()
                                        is PaletteItem.ThemeAction -> item.onSelect()
                                        is PaletteItem.TabAction -> item.onSelect()
                                        is PaletteItem.PaperItem -> item.onSelect()
                                    }
                                }
                                true
                            }
                            Key.Escape -> {
                                onDismissRequest()
                                true
                            }
                            else -> false
                        }
                    } else {
                        false
                    }
                },
        ) {
            Column(Modifier.fillMaxWidth()) {
                // Search Input Header
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 14.dp, vertical = 12.dp),
                ) {
                    Icon(
                        painter = painterResource(Res.drawable.ic_search),
                        contentDescription = "Search",
                        tint = colors.accent,
                        modifier = Modifier.size(20.dp),
                    )
                    Spacer(Modifier.width(10.dp))
                    BasicTextField(
                        value = query,
                        onValueChange = {
                            query = it
                            selectedIndex = 0
                        },
                        textStyle = MaterialTheme.typography.bodyLarge.copy(
                            color = colors.fillPrimary,
                            fontSize = 15.sp,
                        ),
                        cursorBrush = SolidColor(colors.accent),
                        singleLine = true,
                        modifier = Modifier
                            .weight(1f)
                            .focusRequester(focusRequester),
                        decorationBox = { innerTextField ->
                            if (query.isEmpty()) {
                                Text(
                                    "Type a command, paper title, or action...",
                                    style = MaterialTheme.typography.bodyLarge.copy(
                                        color = colors.fillTertiary,
                                        fontSize = 15.sp,
                                    ),
                                )
                            }
                            innerTextField()
                        },
                    )
                    if (query.isNotEmpty()) {
                        Box(
                            contentAlignment = Alignment.Center,
                            modifier = Modifier
                                .size(22.dp)
                                .clip(CircleShape)
                                .background(colors.fillQuarternary)
                                .clickable { query = ""; selectedIndex = 0 },
                        ) {
                            Icon(
                                painterResource(Res.drawable.ic_close),
                                contentDescription = "Clear",
                                tint = colors.fillSecondary,
                                modifier = Modifier.size(14.dp),
                            )
                        }
                        Spacer(Modifier.width(8.dp))
                    }
                    // Esc Badge
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(4.dp))
                            .background(colors.fillQuarternary)
                            .padding(horizontal = 6.dp, vertical = 2.dp),
                    ) {
                        Text(
                            "ESC",
                            style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.SemiBold),
                            color = colors.fillSecondary,
                        )
                    }
                }

                // Category Filter Chips
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 14.dp, vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    PaletteCategory.entries.forEach { cat ->
                        val isCatSelected = selectedCategory == cat
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(Dimens.RadiusFull))
                                .background(if (isCatSelected) colors.accent else colors.fillQuarternary.copy(alpha = 0.5f))
                                .clickable {
                                    selectedCategory = cat
                                    selectedIndex = 0
                                }
                                .padding(horizontal = 10.dp, vertical = 3.dp),
                        ) {
                            Text(
                                cat.title,
                                style = MaterialTheme.typography.labelSmall.copy(
                                    fontWeight = if (isCatSelected) FontWeight.Bold else FontWeight.Medium,
                                    fontSize = 11.sp,
                                ),
                                color = if (isCatSelected) colors.onAccent else colors.fillSecondary,
                            )
                        }
                    }
                }

                HorizontalDivider(color = colors.border.copy(alpha = 0.4f), modifier = Modifier.padding(top = 8.dp))

                // Results List
                LazyColumn(
                    state = listState,
                    contentPadding = PaddingValues(vertical = 6.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .heightIn(max = 380.dp),
                ) {
                    if (allItems.isEmpty()) {
                        item {
                            Box(
                                contentAlignment = Alignment.Center,
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(vertical = 36.dp),
                            ) {
                                Text(
                                    "No matching commands or papers found",
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = colors.fillTertiary,
                                )
                            }
                        }
                    } else {
                        itemsIndexed(allItems) { index, item ->
                            val isHighlighted = index == selectedIndex
                            PaletteItemRow(
                                item = item,
                                isHighlighted = isHighlighted,
                                onClick = {
                                    when (item) {
                                        is PaletteItem.Action -> item.onExecute()
                                        is PaletteItem.ThemeAction -> item.onSelect()
                                        is PaletteItem.TabAction -> item.onSelect()
                                        is PaletteItem.PaperItem -> item.onSelect()
                                    }
                                },
                            )
                        }
                    }
                }

                HorizontalDivider(color = colors.border.copy(alpha = 0.4f))

                // Footer Hint Bar
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(colors.tabbar.copy(alpha = 0.5f))
                        .padding(horizontal = 14.dp, vertical = 8.dp),
                ) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        FooterShortcutHint("↑↓", "Navigate")
                        FooterShortcutHint("↵", "Open / Select")
                        FooterShortcutHint("esc", "Dismiss")
                    }
                    Text(
                        "${allItems.size} items",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp),
                        color = colors.fillTertiary,
                    )
                }
            }
        }
    }

    LaunchedEffect(Unit) {
        focusRequester.requestFocus()
    }
}

@Composable
private fun PaletteItemRow(
    item: PaletteItem,
    isHighlighted: Boolean,
    onClick: () -> Unit,
) {
    val colors = QuireTheme.colors
    val bg = if (isHighlighted) colors.accent.copy(alpha = 0.15f) else Color.Transparent
    val borderModifier = if (isHighlighted) Modifier.border(1.dp, colors.accent.copy(alpha = 0.35f), RoundedCornerShape(Dimens.RadiusSmall)) else Modifier

    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 8.dp, vertical = 2.dp)
            .clip(RoundedCornerShape(Dimens.RadiusSmall))
            .background(bg)
            .then(borderModifier)
            .clickable(onClick = onClick)
            .padding(horizontal = 10.dp, vertical = 7.dp),
    ) {
        // Category Icon or Color Pill
        when (item) {
            is PaletteItem.ThemeAction -> {
                Box(
                    modifier = Modifier
                        .size(14.dp)
                        .clip(CircleShape)
                        .background(dev.neurokitchen.quire.presentation.designsystem.quireColors(item.theme).accent),
                )
            }
            is PaletteItem.Action -> {
                val icon = when (item.iconType) {
                    ActionIcon.Settings -> Res.drawable.ic_settings
                    ActionIcon.Sync -> Res.drawable.ic_sync
                    ActionIcon.Tools -> Res.drawable.ic_inventory_2
                    ActionIcon.Library -> Res.drawable.ic_library_books
                    else -> Res.drawable.ic_description
                }
                Icon(
                    painter = painterResource(icon),
                    contentDescription = null,
                    tint = if (isHighlighted) colors.accent else colors.fillSecondary,
                    modifier = Modifier.size(16.dp),
                )
            }
            is PaletteItem.TabAction -> {
                Icon(
                    painter = painterResource(Res.drawable.ic_description),
                    contentDescription = null,
                    tint = colors.accent,
                    modifier = Modifier.size(16.dp),
                )
            }
            is PaletteItem.PaperItem -> {
                Icon(
                    painter = painterResource(Res.drawable.ic_description),
                    contentDescription = null,
                    tint = if (isHighlighted) colors.accent else colors.fillSecondary,
                    modifier = Modifier.size(16.dp),
                )
            }
        }

        Spacer(Modifier.width(10.dp))

        // Titles
        Column(Modifier.weight(1f)) {
            Text(
                text = item.title,
                style = MaterialTheme.typography.bodyMedium.copy(
                    fontWeight = if (isHighlighted) FontWeight.SemiBold else FontWeight.Normal,
                    fontSize = 13.sp,
                ),
                color = if (isHighlighted) colors.fillPrimary else colors.fillPrimary.copy(alpha = 0.9f),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (!item.subtitle.isNullOrBlank()) {
                Text(
                    text = item.subtitle ?: "",
                    style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp),
                    color = colors.fillTertiary,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }

        Spacer(Modifier.width(8.dp))

        // Category Tag or Shortcut Pill
        if (item is PaletteItem.Action && item.shortcut != null) {
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(3.dp))
                    .background(colors.fillQuarternary)
                    .padding(horizontal = 5.dp, vertical = 2.dp),
            ) {
                Text(
                    item.shortcut,
                    style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.Medium),
                    color = colors.fillSecondary,
                )
            }
        } else {
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(3.dp))
                    .background(colors.fillQuarternary.copy(alpha = 0.5f))
                    .padding(horizontal = 5.dp, vertical = 2.dp),
            ) {
                Text(
                    item.category.title.uppercase(),
                    style = MaterialTheme.typography.labelSmall.copy(fontSize = 9.sp, fontWeight = FontWeight.SemiBold),
                    color = colors.fillTertiary,
                )
            }
        }
    }
}

@Composable
private fun FooterShortcutHint(badge: String, label: String) {
    val colors = QuireTheme.colors
    Row(verticalAlignment = Alignment.CenterVertically) {
        Box(
            modifier = Modifier
                .clip(RoundedCornerShape(3.dp))
                .background(colors.fillQuarternary)
                .padding(horizontal = 4.dp, vertical = 1.dp),
        ) {
            Text(
                badge,
                style = MaterialTheme.typography.labelSmall.copy(fontSize = 9.sp, fontWeight = FontWeight.Bold),
                color = colors.fillSecondary,
            )
        }
        Spacer(Modifier.width(4.dp))
        Text(
            label,
            style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp),
            color = colors.fillTertiary,
        )
    }
}
