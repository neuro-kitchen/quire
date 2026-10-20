package dev.neurokitchen.quire.presentation.screens.library.components

import dev.neurokitchen.quire.presentation.screens.library.NameDialog
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_chevron_right
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_library_books
import dev.neurokitchen.quire.resources.library_expand
import dev.neurokitchen.quire.resources.library_my_library
import dev.neurokitchen.quire.resources.library_new_project
import dev.neurokitchen.quire.resources.library_trash
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * VS Code-style Activity Rail displayed when the collections sidebar is minimized.
 * Width: 48.dp. Provides fast access to main library collections and sidebar expansion.
 */
@Composable
fun ActivityRail(
    state: LibraryState,
    onEvent: (LibraryEvent) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = modifier
            .width(Dimens.ActivityRailWidth)
            .fillMaxHeight()
            .background(colors.sidepane)
            .padding(vertical = Dimens.SpaceXs),
    ) {
        // Toggle/Expand button
        RailButton(
            icon = Res.drawable.ic_chevron_right,
            description = stringResource(Res.string.library_expand) + " (Ctrl+B)",
            isSelected = false,
            onClick = { onEvent(LibraryEvent.OnToggleSidebar) },
        )

        Spacer(Modifier.height(Dimens.SpaceSm))

        // All References
        RailButton(
            icon = Res.drawable.ic_library_books,
            description = stringResource(Res.string.library_my_library),
            isSelected = state.source == LibrarySource.All,
            onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.All)) },
        )

        // New Collection
        RailButton(
            icon = Res.drawable.ic_create_new_folder,
            description = stringResource(Res.string.library_new_project),
            isSelected = false,
            onClick = { onEvent(LibraryEvent.OnNameDialog(NameDialog.NewProject())) },
        )

        // Trash
        RailButton(
            icon = Res.drawable.ic_delete,
            description = stringResource(Res.string.library_trash),
            isSelected = state.source == LibrarySource.Trash,
            onClick = { onEvent(LibraryEvent.OnSourceClick(LibrarySource.Trash)) },
        )

        Spacer(Modifier.weight(1f))

        // Item count pill
        Box(
            modifier = Modifier
                .clip(RoundedCornerShape(Dimens.RadiusFull))
                .background(colors.fillQuarternary)
                .padding(horizontal = 6.dp, vertical = 2.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = "${state.references.size}",
                style = MaterialTheme.typography.labelSmall,
                color = colors.fillSecondary,
            )
        }
    }
}

@Composable
private fun RailButton(
    icon: DrawableResource,
    description: String,
    isSelected: Boolean,
    onClick: () -> Unit,
) {
    val colors = QuireTheme.colors
    Box(
        contentAlignment = Alignment.Center,
        modifier = Modifier
            .size(Dimens.ToolbarButton)
            .clip(RoundedCornerShape(Dimens.RadiusMedium))
            .background(if (isSelected) colors.accent.copy(alpha = 0.15f) else androidx.compose.ui.graphics.Color.Transparent)
            .clickable(onClick = onClick)
            .padding(4.dp),
    ) {
        Icon(
            painterResource(icon),
            contentDescription = description,
            tint = if (isSelected) colors.accent else colors.fillSecondary,
            modifier = Modifier.size(Dimens.Icon),
        )
    }
}
