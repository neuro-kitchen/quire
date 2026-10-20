package dev.neurokitchen.quire.presentation.components.notice

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.notice_dismiss
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

/** The outcome of an action: done (a message with its arguments) or failed (the engine's message). */
sealed interface Notice {
    data class Done(val message: StringResource, val args: List<Any> = emptyList()) : Notice
    data class Failed(val message: String) : Notice
}

/** What the last action did, or the engine's message when it failed, until dismissed. */
@Composable
fun NoticeBar(notice: Notice, onDismiss: () -> Unit, modifier: Modifier = Modifier) {
    val failed = notice is Notice.Failed
    Surface(
        shape = RoundedCornerShape(Dimens.RadiusBase),
        color = if (failed) MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.inverseSurface,
        tonalElevation = Dimens.ElevationMedium,
        modifier = modifier.widthIn(max = 640.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(start = Dimens.SpaceMd)) {
            Text(
                text = when (notice) {
                    is Notice.Done -> stringResource(notice.message, *notice.args.toTypedArray())
                    is Notice.Failed -> notice.message
                },
                style = MaterialTheme.typography.bodyMedium,
                color = if (failed) MaterialTheme.colorScheme.onErrorContainer else MaterialTheme.colorScheme.inverseOnSurface,
                modifier = Modifier.weight(1f, fill = false),
            )
            TextButton(onClick = onDismiss) { Text(stringResource(Res.string.notice_dismiss)) }
        }
    }
}
