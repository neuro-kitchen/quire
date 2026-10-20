package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.ToolbarIconButton
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_chevron_right
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_draft
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.library_collapse
import dev.neurokitchen.quire.resources.library_expand
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_new_project
import dev.neurokitchen.quire.resources.library_trash
import dev.neurokitchen.quire.resources.library_unfiled
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/** Zotero's collections pane: the library, its projects and the tag selector below. */
@Composable
fun ProjectsPane(
    state: LibraryState,
    onEvent: (LibraryEvent) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier.fillMaxHeight().background(QuireTheme.colors.sidepane)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().height(Dimens.ToolbarHeight).padding(horizontal = Dimens.SpaceXs),
        ) {
            ToolbarIconButton(Res.drawable.ic_create_new_folder, stringResource(Res.string.library_new_project), onClick = {})
        }
        LazyColumn(Modifier.weight(1f).padding(horizontal = Dimens.SpaceXs)) {
            item(key = "all") {
                SourceRow(
                    icon = Res.drawable.ic_library_books,
                    name = stringResource(Res.string.library_my_library),
                    depth = 0,
                    isSelected = state.source == LibrarySource.All,
                    onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.All)) },
                    twisty = { Spacer(Modifier.width(TwistyWidth)) },
                )
            }
            items(state.projects, key = { it.id }) { project ->
                SourceRow(
                    icon = Res.drawable.ic_folder,
                    name = project.name,
                    depth = project.depth + 1,
                    isSelected = project.isSelected,
                    onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject(project.id))) },
                    twisty = {
                        if (project.hasChildren) {
                            Twisty(
                                expanded = project.isExpanded,
                                selected = project.isSelected,
                                onClick = { onEvent(LibraryEvent.OnProjectExpandToggle(project.id)) },
                            )
                        } else {
                            Spacer(Modifier.width(TwistyWidth))
                        }
                    },
                )
            }
            item(key = "unfiled") {
                SourceRow(
                    icon = Res.drawable.ic_draft,
                    name = stringResource(Res.string.library_unfiled),
                    depth = 1,
                    isSelected = state.source == LibrarySource.Unfiled,
                    onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.Unfiled)) },
                    twisty = { Spacer(Modifier.width(TwistyWidth)) },
                )
            }
            item(key = "trash") {
                SourceRow(
                    icon = Res.drawable.ic_delete,
                    name = stringResource(Res.string.library_trash),
                    depth = 1,
                    isSelected = state.source == LibrarySource.Trash,
                    onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.Trash)) },
                    twisty = { Spacer(Modifier.width(TwistyWidth)) },
                )
            }
        }
        HorizontalDivider(color = QuireTheme.colors.paneDivider)
        TagSelector(
            tags = state.tags,
            onTagClick = { onEvent(LibraryEvent.OnTagClick(it)) },
            modifier = Modifier.fillMaxWidth().height(TagSelectorHeight),
        )
    }
}

private val TwistyWidth = 16.dp
private val IndentPerLevel = 16.dp
private val TagSelectorHeight = 200.dp

@Composable
private fun SourceRow(
    icon: DrawableResource,
    name: String,
    depth: Int,
    isSelected: Boolean,
    onClick: () -> Unit,
    twisty: @Composable () -> Unit,
) {
    val colors = QuireTheme.colors
    val content = if (isSelected) colors.onAccent else colors.fillPrimary
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .height(Dimens.RowHeight)
            .clip(RoundedCornerShape(Dimens.RadiusLarge))
            .background(if (isSelected) colors.accent else Color.Transparent)
            .clickable(onClick = onClick)
            .padding(start = IndentPerLevel * depth, end = Dimens.SpaceXs),
    ) {
        twisty()
        Icon(
            painterResource(icon), null,
            tint = if (isSelected) colors.onAccent else colors.accent,
            modifier = Modifier.padding(horizontal = Dimens.SpaceMin).size(Dimens.Icon),
        )
        Text(name, style = MaterialTheme.typography.bodyMedium, color = content, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
private fun Twisty(expanded: Boolean, selected: Boolean, onClick: () -> Unit) {
    val description = stringResource(if (expanded) Res.string.library_collapse else Res.string.library_expand)
    Box(Modifier.size(TwistyWidth).clickable(onClick = onClick), contentAlignment = Alignment.Center) {
        Icon(
            painterResource(Res.drawable.ic_chevron_right), description,
            tint = if (selected) QuireTheme.colors.onAccent else QuireTheme.colors.fillSecondary,
            modifier = Modifier.size(14.dp).rotate(if (expanded) 90f else 0f),
        )
    }
}
