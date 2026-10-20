package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.library_search_hint
import org.jetbrains.compose.resources.stringResource

@Composable
fun LibrarySearchField(
    query: String,
    onQueryChange: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    OutlinedTextField(
        value = query,
        onValueChange = onQueryChange,
        placeholder = { Text(stringResource(Res.string.library_search_hint)) },
        singleLine = true,
        modifier = modifier.fillMaxWidth(),
    )
}
