package dev.neurokitchen.quire.presentation.screens.library.components

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
 * Zotero's tag selector: the tags of the references in view as a wrapped list; clicking one keeps
 * only references with it. The filter field narrows the list itself (local UI state).
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TagSelector(
    tags: List<TagState>,
    onTagClick: (String) -> Unit,
    modifier: Modifier = Modifier,
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
                    shown.forEach { tag -> Tag(tag, onClick = { onTagClick(tag.name) }) }
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
private fun Tag(tag: TagState, onClick: () -> Unit) {
    val colors = QuireTheme.colors
    Text(
        text = tag.name,
        style = MaterialTheme.typography.bodyMedium,
        color = if (tag.isSelected) colors.onAccent else colors.fillPrimary,
        modifier = Modifier
            .clip(RoundedCornerShape(Dimens.RadiusBase))
            .background(if (tag.isSelected) colors.accent else Color.Transparent)
            .clickable(onClick = onClick)
            .padding(horizontal = 4.dp, vertical = 1.dp),
    )
}
