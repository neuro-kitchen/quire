package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.CompactTextField
import dev.neurokitchen.quire.presentation.designsystem.components.ToolbarIconButton
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_add
import dev.neurokitchen.quire.resources.ic_attach_file
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.ic_sticky_note_2
import dev.neurokitchen.quire.resources.library_add_attachment
import dev.neurokitchen.quire.resources.library_add_by_identifier
import dev.neurokitchen.quire.resources.library_new_item
import dev.neurokitchen.quire.resources.library_new_note
import dev.neurokitchen.quire.resources.library_search_hint
import org.jetbrains.compose.resources.stringResource

/** Zotero's items toolbar: new item, add by identifier, note, attachment, and the search field. */
@Composable
fun ItemsToolbar(
    query: String,
    onQueryChange: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(2.dp),
        modifier = modifier
            .fillMaxWidth()
            .height(Dimens.ToolbarHeight)
            .background(QuireTheme.colors.toolbar)
            .padding(horizontal = Dimens.SpaceXs),
    ) {
        // The actions arrive with the engine bridge; the buttons show where they will be.
        ToolbarIconButton(Res.drawable.ic_add, stringResource(Res.string.library_new_item), onClick = {})
        ToolbarIconButton(Res.drawable.ic_auto_fix_high, stringResource(Res.string.library_add_by_identifier), onClick = {})
        ToolbarIconButton(Res.drawable.ic_sticky_note_2, stringResource(Res.string.library_new_note), onClick = {})
        ToolbarIconButton(Res.drawable.ic_attach_file, stringResource(Res.string.library_add_attachment), onClick = {})
        Spacer(Modifier.weight(1f))
        CompactTextField(
            value = query,
            onValueChange = onQueryChange,
            placeholder = stringResource(Res.string.library_search_hint),
            leadingIcon = Res.drawable.ic_search,
            modifier = Modifier.width(SearchWidth),
        )
    }
}

private val SearchWidth = 240.dp
