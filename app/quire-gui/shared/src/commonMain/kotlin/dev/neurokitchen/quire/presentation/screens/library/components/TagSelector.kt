package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.runtime.remember
import dev.neurokitchen.quire.presentation.designsystem.components.onSecondaryClick
import dev.neurokitchen.quire.resources.menu_delete
import dev.neurokitchen.quire.resources.menu_rename
import dev.neurokitchen.quire.resources.menu_tag_color
import dev.neurokitchen.quire.resources.menu_tag_no_color
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.CompactTextField
import dev.neurokitchen.quire.presentation.screens.library.TagState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.tags_filter
import dev.neurokitchen.quire.resources.tags_none
import org.jetbrains.compose.resources.stringResource

/**
 * Zotero's tag selector: the library's coloured tags first (with their colour), then the tags of
 * the references in view, as a wrapped list; clicking one keeps only references with it, a
 * secondary click offers rename, colour and delete. The filter field narrows the list itself
 * (local UI state).
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TagSelector(
    tags: List<TagState>,
    onTagClick: (String) -> Unit,
    modifier: Modifier = Modifier,
    onRename: (String) -> Unit = {},
    /** A tag and its new colour (`#rrggbb`), or `null` to take the colour away. */
    onColor: (String, String?) -> Unit = { _, _ -> },
    onDelete: (String) -> Unit = {},
) {
    var filter by rememberSaveable { mutableStateOf("") }
    val shown = tags.filter { it.isSelected || filter.isBlank() || filter.lowercase() in it.name.lowercase() }
    Column(modifier.padding(Dimens.SpaceXs)) {
        Box(Modifier.weight(1f).fillMaxWidth()) {
            if (shown.isEmpty()) {
                Text(
                    stringResource(Res.string.tags_none),
                    style = MaterialTheme.typography.bodyMedium,
                    color = QuireTheme.colors.fillSecondary,
                    modifier = Modifier.align(Alignment.Center),
                )
            } else {
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(2.dp),
                    verticalArrangement = Arrangement.spacedBy(2.dp),
                    modifier = Modifier.verticalScroll(rememberScrollState()),
                ) {
                    shown.forEach { tag ->
                        Tag(
                            tag,
                            onClick = { onTagClick(tag.name) },
                            onRename = { onRename(tag.name) },
                            onColor = { onColor(tag.name, it) },
                            onDelete = { onDelete(tag.name) },
                        )
                    }
                }
            }
        }
        CompactTextField(
            value = filter,
            onValueChange = { filter = it },
            placeholder = stringResource(Res.string.tags_filter),
            leadingIcon = Res.drawable.ic_search,
            modifier = Modifier.fillMaxWidth().padding(top = Dimens.SpaceXs),
        )
    }
}

@Composable
private fun Tag(tag: TagState, onClick: () -> Unit, onRename: () -> Unit, onColor: (String?) -> Unit, onDelete: () -> Unit) {
    val colors = QuireTheme.colors
    var menuOpen by remember { mutableStateOf(false) }
    val tagColor = tag.color?.let(::parseHex)
    Box {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .clip(RoundedCornerShape(Dimens.RadiusBase))
                .background(if (tag.isSelected) colors.accent else Color.Transparent)
                .onSecondaryClick { menuOpen = true }
                .clickable(onClick = onClick)
                .padding(horizontal = Dimens.SpaceMin, vertical = 1.dp),
        ) {
            if (tagColor != null) {
                Box(Modifier.padding(end = Dimens.SpaceMin).size(8.dp).clip(CircleShape).background(tagColor))
            }
            Text(
                text = tag.name,
                style = MaterialTheme.typography.bodyMedium,
                color = when {
                    tag.isSelected -> colors.onAccent
                    tagColor != null -> tagColor
                    else -> colors.fillPrimary
                },
            )
        }
        DropdownMenu(expanded = menuOpen, onDismissRequest = { menuOpen = false }) {
            DropdownMenuItem(text = { Text(stringResource(Res.string.menu_rename)) }, onClick = { menuOpen = false; onRename() })
            Text(
                stringResource(Res.string.menu_tag_color),
                style = MaterialTheme.typography.labelSmall,
                color = colors.fillSecondary,
                modifier = Modifier.padding(horizontal = Dimens.SpaceSm, vertical = Dimens.SpaceMin),
            )
            Row(
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.padding(horizontal = Dimens.SpaceSm, vertical = Dimens.SpaceMin),
            ) {
                TagColors.forEach { hex ->
                    Box(
                        Modifier
                            .size(18.dp)
                            .clip(CircleShape)
                            .background(parseHex(hex))
                            .then(if (hex.equals(tag.color, ignoreCase = true)) Modifier.border(2.dp, colors.fillPrimary, CircleShape) else Modifier)
                            .clickable { menuOpen = false; onColor(hex) },
                    )
                }
            }
            if (tag.color != null) {
                DropdownMenuItem(text = { Text(stringResource(Res.string.menu_tag_no_color)) }, onClick = { menuOpen = false; onColor(null) })
            }
            DropdownMenuItem(
                text = { Text(stringResource(Res.string.menu_delete), color = MaterialTheme.colorScheme.error) },
                onClick = { menuOpen = false; onDelete() },
            )
        }
    }
}

/** Zotero's tag colours. */
private val TagColors = listOf("#FF6666", "#FF8C19", "#999999", "#5FB236", "#009980", "#2EA8E5", "#576DD9", "#A28AE5", "#A6507B")

/** `#rrggbb` as a colour; grey if it is not one. */
private fun parseHex(hex: String): Color =
    hex.removePrefix("#").takeIf { it.length == 6 }?.toLongOrNull(16)?.let { Color(0xFF000000 or it) } ?: Color.Gray
