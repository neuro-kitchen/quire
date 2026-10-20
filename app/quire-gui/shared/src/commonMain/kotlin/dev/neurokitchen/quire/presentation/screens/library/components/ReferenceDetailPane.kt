package dev.neurokitchen.quire.presentation.screens.library.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.screens.library.ReferenceDetailState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.library_close_detail
import org.jetbrains.compose.resources.stringResource

/** The selected reference on narrow windows, in place of the list. */
@Composable
fun ReferenceDetailPane(
    reference: ReferenceDetailState,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(24.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = reference.citeKey,
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.weight(1f),
            )
            TextButton(onClick = onClose) { Text(stringResource(Res.string.library_close_detail)) }
        }
        Spacer(Modifier.height(8.dp))
        Text(reference.title, style = MaterialTheme.typography.headlineSmall)
        Spacer(Modifier.height(8.dp))
        Text(
            text = listOf(reference.creator, reference.year).filter { it.isNotEmpty() }.joinToString(" · "),
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        if (reference.abstract.isNotEmpty()) {
            Spacer(Modifier.height(16.dp))
            Text(reference.abstract, style = MaterialTheme.typography.bodyMedium)
        }
    }
}
