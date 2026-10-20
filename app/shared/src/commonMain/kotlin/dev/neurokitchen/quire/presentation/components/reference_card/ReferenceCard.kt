package dev.neurokitchen.quire.presentation.components.reference_card

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

/** One reference in a list: title, authors and year, and its cite key. */
@Composable
fun ReferenceCard(
    state: ReferenceCardState,
    onEvent: (ReferenceCardEvent) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    Card(
        onClick = { onEvent(ReferenceCardEvent.OnClick(state.id)) },
        colors = CardDefaults.cardColors(
            containerColor = if (state.isSelected) colors.secondaryContainer else colors.surfaceContainerLow,
        ),
        modifier = modifier.fillMaxWidth(),
    ) {
        Column(Modifier.padding(16.dp)) {
            Text(
                text = state.title,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(4.dp))
            Row {
                Text(
                    text = listOf(state.creator, state.year).filter { it.isNotEmpty() }.joinToString(" · "),
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.onSurfaceVariant,
                    modifier = Modifier.weight(1f),
                )
                Text(
                    text = state.citeKey,
                    style = MaterialTheme.typography.labelMedium,
                    color = colors.primary,
                )
            }
        }
    }
}
