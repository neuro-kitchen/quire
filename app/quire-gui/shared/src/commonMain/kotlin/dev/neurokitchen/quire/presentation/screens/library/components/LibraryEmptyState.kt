package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.library_empty
import dev.neurokitchen.quire.resources.library_no_match
import org.jetbrains.compose.resources.stringResource

/** Shown instead of the list: an empty library, or a search with no match. */
@Composable
fun LibraryEmptyState(query: String, modifier: Modifier = Modifier) {
    Box(modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Text(
            text = if (query.isBlank()) stringResource(Res.string.library_empty)
            else stringResource(Res.string.library_no_match, query),
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
