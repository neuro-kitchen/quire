package dev.neurokitchen.quire.presentation.screens.library.components

import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.menu_remove
import androidx.compose.foundation.border
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.layout.boundsInWindow
import androidx.compose.ui.layout.onGloballyPositioned
import dev.neurokitchen.quire.presentation.designsystem.components.onSecondaryClick
import dev.neurokitchen.quire.presentation.platform.pickSaveFile
import dev.neurokitchen.quire.presentation.screens.library.Confirm
import dev.neurokitchen.quire.presentation.screens.library.NameDialog
import dev.neurokitchen.quire.resources.menu_delete
import dev.neurokitchen.quire.resources.menu_new_subproject
import dev.neurokitchen.quire.resources.menu_rename
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
import dev.neurokitchen.quire.resources.ic_auto_fix_high
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
    /** References dragged from the table; a project row under the pointer takes them. */
    drag: ReferenceDrag? = null,
) {
    Column(modifier.fillMaxHeight().background(QuireTheme.colors.sidepane)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().height(Dimens.ToolbarHeight).padding(horizontal = Dimens.SpaceXs),
        ) {
            ToolbarIconButton(
                Res.drawable.ic_create_new_folder,
                stringResource(Res.string.library_new_project),
                onClick = { onEvent(LibraryEvent.OnNameDialog(NameDialog.NewProject())) },
            )
            ToolbarIconButton(
                Res.drawable.ic_auto_fix_high,
                "New Smart Collection...",
                onClick = {
                    val q = if (state.query.isNotBlank()) state.query else "status:unread"
                    onEvent(LibraryEvent.OnNameDialog(NameDialog.SaveSearch(q)))
                },
            )
            Spacer(Modifier.weight(1f))
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .size(Dimens.ToolbarButton)
                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                    .clickable { onEvent(LibraryEvent.OnToggleSidebar) },
            ) {
                Icon(
                    painterResource(Res.drawable.ic_chevron_right),
                    contentDescription = stringResource(Res.string.library_collapse) + " (Ctrl+B)",
                    tint = QuireTheme.colors.fillSecondary,
                    modifier = Modifier.size(16.dp).rotate(180f),
                )
            }
        }
        LazyColumn(Modifier.weight(1f).padding(horizontal = Dimens.SpaceXs)) {
            item(key = "all") {
                SourceRow(
                    icon = Res.drawable.ic_library_books,
                    name = stringResource(Res.string.library_my_library),
                    depth = 0,
                    isSelected = state.source == LibrarySource.All,
                    onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.All)) },
                    count = state.references.size,
                    twisty = { Spacer(Modifier.width(TwistyWidth)) },
                )
            }
            items(state.projects, key = { it.id }) { project ->
                var menuOpen by remember { mutableStateOf(false) }
                if (drag != null) {
                    DisposableEffect(project.id) { onDispose { drag.unregister(project.id) } }
                }
                Box {
                    SourceRow(
                        modifier = Modifier
                            .onSecondaryClick { menuOpen = true }
                            .then(if (drag != null) Modifier.onGloballyPositioned { drag.register(project.id, it.boundsInWindow()) } else Modifier),
                        isDropTarget = drag?.target == project.id,
                        icon = Res.drawable.ic_folder,
                        name = project.name,
                        depth = project.depth + 1,
                        isSelected = project.isSelected,
                        onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject(project.id))) },
                        count = project.referenceCount,
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
                    ProjectMenu(menuOpen, onDismiss = { menuOpen = false }) { action ->
                        menuOpen = false
                        when (action) {
                            ProjectAction.NewSubproject -> onEvent(LibraryEvent.OnNameDialog(NameDialog.NewProject(parentPath = project.id)))
                            ProjectAction.Rename -> onEvent(LibraryEvent.OnNameDialog(NameDialog.RenameProject(project.id, project.name)))
                            ProjectAction.Delete -> onEvent(LibraryEvent.OnConfirm(Confirm.DeleteProject(project.id, project.name)))
                            ProjectAction.ExportBib -> {
                                val cleanName = project.name.lowercase().replace(" ", "_").filter { it.isLetterOrDigit() || it == '_' }
                                pickSaveFile("Export ${project.name} as BibTeX", "$cleanName.bib")?.let { path ->
                                    onEvent(LibraryEvent.OnExportProject(project.id, path))
                                }
                            }
                        }
                    }
                }
            }
            items(state.savedSearches, key = { "search:" + it.name }) { saved ->
                var menuOpen by remember { mutableStateOf(false) }
                Box {
                    SourceRow(
                        modifier = Modifier.onSecondaryClick { menuOpen = true },
                        icon = Res.drawable.ic_auto_fix_high,
                        name = saved.name,
                        depth = 1,
                        isSelected = saved.isActive,
                        onClick = { onEvent(LibraryEvent.OnSavedSearchClick(saved.name)) },
                        twisty = { Spacer(Modifier.width(TwistyWidth)) },
                    )
                    DropdownMenu(expanded = menuOpen, onDismissRequest = { menuOpen = false }) {
                        DropdownMenuItem(
                            text = { Text(stringResource(Res.string.menu_rename)) },
                            onClick = { menuOpen = false; onEvent(LibraryEvent.OnNameDialog(NameDialog.RenameSavedSearch(saved.name))) },
                        )
                        DropdownMenuItem(
                            text = { Text(stringResource(Res.string.menu_remove), color = MaterialTheme.colorScheme.error) },
                            onClick = { menuOpen = false; onEvent(LibraryEvent.OnSavedSearchRemove(saved.name)) },
                        )
                    }
                }
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
            onRename = { onEvent(LibraryEvent.OnNameDialog(NameDialog.RenameTag(it))) },
            onColor = { name, color -> onEvent(LibraryEvent.OnTagColor(name, color)) },
            onDelete = { onEvent(LibraryEvent.OnConfirm(Confirm.DeleteTag(it))) },
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
    count: Int? = null,
    modifier: Modifier = Modifier,
    /** References are dragged over it. */
    isDropTarget: Boolean = false,
    twisty: @Composable () -> Unit,
) {
    val colors = QuireTheme.colors
    val content = if (isSelected) colors.onAccent else colors.fillPrimary
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier
            .fillMaxWidth()
            .height(Dimens.RowHeight)
            .clip(RoundedCornerShape(Dimens.RadiusLarge))
            .background(
                when {
                    isSelected -> colors.accent
                    isDropTarget -> colors.accent.copy(alpha = 0.2f)
                    else -> Color.Transparent
                },
            )
            .then(if (isDropTarget) Modifier.border(1.dp, colors.accent, RoundedCornerShape(Dimens.RadiusLarge)) else Modifier)
            .clickable(onClick = onClick)
            .padding(start = IndentPerLevel * depth, end = Dimens.SpaceXs),
    ) {
        twisty()
        Icon(
            painterResource(icon), null,
            tint = if (isSelected) colors.onAccent else colors.accent,
            modifier = Modifier.padding(horizontal = Dimens.SpaceMin).size(Dimens.Icon),
        )
        Text(name, style = MaterialTheme.typography.bodyMedium, color = content, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
        if (count != null && count > 0) {
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(Dimens.RadiusFull))
                    .background(if (isSelected) colors.onAccent.copy(alpha = 0.2f) else colors.fillQuinary)
                    .padding(horizontal = 6.dp, vertical = 1.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = count.toString(),
                    style = MaterialTheme.typography.labelSmall,
                    color = if (isSelected) colors.onAccent else colors.fillSecondary,
                )
            }
        }
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

private enum class ProjectAction { NewSubproject, Rename, ExportBib, Delete }

@Composable
private fun ProjectMenu(expanded: Boolean, onDismiss: () -> Unit, onAction: (ProjectAction) -> Unit) {
    DropdownMenu(expanded = expanded, onDismissRequest = onDismiss) {
        DropdownMenuItem(text = { Text(stringResource(Res.string.menu_new_subproject)) }, onClick = { onAction(ProjectAction.NewSubproject) })
        DropdownMenuItem(text = { Text(stringResource(Res.string.menu_rename)) }, onClick = { onAction(ProjectAction.Rename) })
        DropdownMenuItem(
            text = { Text("Export as BibTeX (.bib)...") },
            onClick = { onAction(ProjectAction.ExportBib) },
        )
        DropdownMenuItem(
            text = { Text(stringResource(Res.string.menu_delete), color = MaterialTheme.colorScheme.error) },
            onClick = { onAction(ProjectAction.Delete) },
        )
    }
}
