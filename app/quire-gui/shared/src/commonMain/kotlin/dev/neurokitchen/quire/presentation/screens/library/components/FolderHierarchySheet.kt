package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.SheetState
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.ProjectRowState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_close
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_draft
import dev.neurokitchen.quire.resources.ic_folder
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_new_collection
import dev.neurokitchen.quire.resources.library_trash
import dev.neurokitchen.quire.resources.library_unfiled
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * Mobile Library Hierarchy Sheet:
 * Displays a clean, indented tree structure of collections and projects.
 * Replaces the cramped dropdown with a full, thumb-accessible sheet.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FolderHierarchySheet(
    currentSource: LibrarySource,
    projects: List<ProjectRowState>,
    onSelectSource: (LibrarySource) -> Unit,
    onDismiss: () -> Unit,
    onAddProject: (() -> Unit)? = null,
    sheetState: SheetState = rememberModalBottomSheetState(),
) {
    val colors = QuireTheme.colors

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = colors.cardBackground,
        contentColor = colors.fillPrimary,
        tonalElevation = 8.dp,
        shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(bottom = 24.dp),
        ) {
            // Header Row
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp, vertical = 8.dp),
            ) {
                Column {
                    Text(
                        text = "Library",
                        style = MaterialTheme.typography.titleMedium.copy(
                            fontWeight = FontWeight.Bold,
                            fontSize = 18.sp,
                        ),
                        color = colors.fillPrimary,
                    )
                    Text(
                        text = "Browse collections and projects",
                        style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                        color = colors.fillSecondary,
                    )
                }

                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    if (onAddProject != null) {
                        Box(
                            modifier = Modifier
                                .size(36.dp)
                                .clip(CircleShape)
                                .background(colors.fillQuinary)
                                .clickable {
                                    onDismiss()
                                    onAddProject()
                                },
                            contentAlignment = Alignment.Center,
                        ) {
                            Icon(
                                painter = painterResource(Res.drawable.ic_create_new_folder),
                                contentDescription = "New Collection",
                                tint = colors.fillPrimary,
                                modifier = Modifier.size(18.dp),
                            )
                        }
                    }

                    Box(
                        modifier = Modifier
                            .size(36.dp)
                            .clip(CircleShape)
                            .background(colors.fillQuinary)
                            .clickable(onClick = onDismiss),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            painter = painterResource(Res.drawable.ic_close),
                            contentDescription = "Close",
                            tint = colors.fillSecondary,
                            modifier = Modifier.size(18.dp),
                        )
                    }
                }
            }

            HorizontalDivider(
                color = colors.border.copy(alpha = 0.25f),
                modifier = Modifier.padding(vertical = 8.dp),
            )

            // Scrollable tree list
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 12.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                // All References / My Library
                val isAllSelected = currentSource == LibrarySource.All
                HierarchyItem(
                    title = stringResource(Res.string.library_my_library),
                    icon = Res.drawable.ic_library_books,
                    isSelected = isAllSelected,
                    depth = 0,
                    onClick = {
                        onSelectSource(LibrarySource.All)
                        onDismiss()
                    },
                )

                // Unfiled Items
                val isUnfiledSelected = currentSource == LibrarySource.Unfiled
                HierarchyItem(
                    title = stringResource(Res.string.library_unfiled),
                    icon = Res.drawable.ic_draft,
                    isSelected = isUnfiledSelected,
                    depth = 0,
                    onClick = {
                        onSelectSource(LibrarySource.Unfiled)
                        onDismiss()
                    },
                )

                // Projects Section
                if (projects.isNotEmpty()) {
                    Spacer(Modifier.height(8.dp))
                    Text(
                        text = "COLLECTIONS & PROJECTS",
                        style = MaterialTheme.typography.labelSmall.copy(
                            fontWeight = FontWeight.Bold,
                            fontSize = 11.sp,
                            letterSpacing = 0.8.sp,
                        ),
                        color = colors.fillSecondary,
                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 4.dp),
                    )

                    for (project in projects) {
                        val isProjectSelected = currentSource is LibrarySource.InProject && currentSource.path == project.id
                        HierarchyItem(
                            title = project.name,
                            icon = Res.drawable.ic_folder,
                            count = project.referenceCount,
                            isSelected = isProjectSelected,
                            depth = project.depth + 1,
                            onClick = {
                                onSelectSource(LibrarySource.InProject(project.id))
                                onDismiss()
                            },
                        )
                    }

                    if (onAddProject != null) {
                        HierarchyItem(
                            title = stringResource(Res.string.library_new_collection),
                            icon = Res.drawable.ic_create_new_folder,
                            isSelected = false,
                            depth = 1,
                            onClick = {
                                onDismiss()
                                onAddProject()
                            },
                        )
                    }
                }

                // Trash
                Spacer(Modifier.height(8.dp))
                val isTrashSelected = currentSource == LibrarySource.Trash
                HierarchyItem(
                    title = stringResource(Res.string.library_trash),
                    icon = Res.drawable.ic_delete,
                    isSelected = isTrashSelected,
                    depth = 0,
                    onClick = {
                        onSelectSource(LibrarySource.Trash)
                        onDismiss()
                    },
                )
            }
        }
    }
}

@Composable
private fun HierarchyItem(
    title: String,
    icon: org.jetbrains.compose.resources.DrawableResource,
    isSelected: Boolean,
    depth: Int,
    onClick: () -> Unit,
    count: Int? = null,
) {
    val colors = QuireTheme.colors

    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(Dimens.RadiusMedium))
            .background(if (isSelected) colors.accent.copy(alpha = 0.14f) else androidx.compose.ui.graphics.Color.Transparent)
            .clickable(onClick = onClick)
            .padding(start = (depth * 18 + 12).dp, end = 12.dp, top = 10.dp, bottom = 10.dp),
    ) {
        Icon(
            painter = painterResource(icon),
            contentDescription = null,
            tint = if (isSelected) colors.accent else colors.fillSecondary,
            modifier = Modifier.size(20.dp),
        )
        Spacer(Modifier.width(12.dp))
        Text(
            text = title,
            style = MaterialTheme.typography.bodyMedium.copy(
                fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                fontSize = 15.sp,
            ),
            color = if (isSelected) colors.accent else colors.fillPrimary,
            modifier = Modifier.weight(1f),
        )
        if (count != null && count > 0) {
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(Dimens.RadiusFull))
                    .background(colors.fillQuinary)
                    .padding(horizontal = 8.dp, vertical = 2.dp),
            ) {
                Text(
                    text = "$count",
                    style = MaterialTheme.typography.labelSmall.copy(
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                    ),
                    color = colors.fillSecondary,
                )
            }
        }
    }
}
