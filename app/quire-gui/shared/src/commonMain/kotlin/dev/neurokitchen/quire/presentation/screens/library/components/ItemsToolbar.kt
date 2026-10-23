package dev.neurokitchen.quire.presentation.screens.library.components

import dev.neurokitchen.quire.resources.toolbar_toggle_inspector_ctrl_i
import dev.neurokitchen.quire.resources.add_title
import dev.neurokitchen.quire.resources.search_save
import androidx.compose.material3.TextButton
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
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
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.CompactTextField
import dev.neurokitchen.quire.presentation.designsystem.components.ToolbarIconButton
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_add
import dev.neurokitchen.quire.resources.ic_info
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.ic_sticky_note_2
import dev.neurokitchen.quire.resources.library_add_attachment
import dev.neurokitchen.quire.resources.library_new_note
import dev.neurokitchen.quire.resources.library_search_hint
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * Quire items toolbar:
 * - Unified "+ Add Reference" primary action with shortcut indicator [Ctrl+A]
 * - Notes & attachment shortcuts
 * - Search bar with [Ctrl+F] / [Ctrl+S] shortcut and focus requester.
 */
@Composable
fun ItemsToolbar(
    query: String,
    onQueryChange: (String) -> Unit,
    onAddReference: () -> Unit,
    modifier: Modifier = Modifier,
    searchFocusRequester: FocusRequester? = null,
    onToggleInspector: (() -> Unit)? = null,
    isInspectorCollapsed: Boolean = false,
    /** Saves the search as typed; offered while there is one. */
    onSaveSearch: (() -> Unit)? = null,
) {
    val colors = QuireTheme.colors
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
        modifier = modifier
            .fillMaxWidth()
            .height(Dimens.ToolbarHeight)
            .background(colors.toolbar)
            .padding(horizontal = Dimens.SpaceSm),
    ) {
        // One Add button: the dialog has every way in (identifiers, PDFs, files, BibTeX, typed in).

        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .height(30.dp)
                .clip(RoundedCornerShape(Dimens.RadiusBase))
                .background(colors.accent),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .clickable(onClick = onAddReference)
                    .padding(horizontal = 10.dp, vertical = Dimens.SpaceMin),
            ) {
                Icon(
                    painterResource(Res.drawable.ic_add),
                    contentDescription = null,
                    tint = colors.onAccent,
                    modifier = Modifier.size(16.dp),
                )
                Spacer(Modifier.width(6.dp))
                Text(
                    text = stringResource(Res.string.add_title),
                    style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.SemiBold),
                    color = colors.onAccent,
                )
                Spacer(Modifier.width(6.dp))
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(4.dp))
                        .background(colors.onAccent.copy(alpha = 0.2f))
                        .padding(horizontal = Dimens.SpaceMin, vertical = 1.dp),
                ) {
                    Text(
                        text = "Ctrl+N",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 9.sp),
                        color = colors.onAccent,
                    )
                }
            }
        }

        Spacer(Modifier.weight(1f))

        if (onSaveSearch != null && query.isNotBlank()) {
            TextButton(onClick = onSaveSearch, contentPadding = PaddingValues(horizontal = Dimens.SpaceXs), modifier = Modifier.height(30.dp)) {
                Text(stringResource(Res.string.search_save), style = MaterialTheme.typography.labelMedium)
            }
        }

        CompactTextField(
            value = query,
            onValueChange = onQueryChange,
            placeholder = stringResource(Res.string.library_search_hint),
            leadingIcon = Res.drawable.ic_search,
            focusRequester = searchFocusRequester,
            onClear = { onQueryChange("") },
            trailingBadge = {
                Box(
                    modifier = Modifier
                        .clip(RoundedCornerShape(4.dp))
                        .background(colors.fillQuarternary)
                        .padding(horizontal = Dimens.SpaceMin, vertical = 1.dp),
                ) {
                    Text(
                        text = "Ctrl+F",
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 9.sp),
                        color = colors.fillSecondary,
                    )
                }
            },
            modifier = Modifier.width(SearchWidth),
        )

        if (onToggleInspector != null) {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .size(Dimens.ToolbarButton)
                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                    .background(if (!isInspectorCollapsed) colors.accent.copy(alpha = 0.15f) else Color.Transparent)
                    .clickable(onClick = onToggleInspector),
            ) {
                Icon(
                    painterResource(Res.drawable.ic_info),
                    contentDescription = stringResource(Res.string.toolbar_toggle_inspector_ctrl_i),
                    tint = if (!isInspectorCollapsed) colors.accent else colors.fillSecondary,
                    modifier = Modifier.size(16.dp),
                )
            }
        }
    }
}

private val SearchWidth = 260.dp
