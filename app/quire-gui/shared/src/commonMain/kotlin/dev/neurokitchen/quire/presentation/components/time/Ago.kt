package dev.neurokitchen.quire.presentation.components.time

import androidx.compose.runtime.Composable
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.time_days_ago
import dev.neurokitchen.quire.resources.time_hours_ago
import dev.neurokitchen.quire.resources.time_just_now
import dev.neurokitchen.quire.resources.time_minutes_ago
import org.jetbrains.compose.resources.stringResource
import kotlin.time.Clock
import kotlin.time.ExperimentalTime

/** How long ago [millis] (since 1970) was, in words: "just now", "5 min ago", "2 h ago", "3 days ago". */
@OptIn(ExperimentalTime::class)
@Composable
fun ago(millis: Long): String {
    val minutes = ((Clock.System.now().toEpochMilliseconds() - millis).coerceAtLeast(0) / 60_000).toInt()
    return when {
        minutes < 1 -> stringResource(Res.string.time_just_now)
        minutes < 60 -> stringResource(Res.string.time_minutes_ago, minutes)
        minutes < 24 * 60 -> stringResource(Res.string.time_hours_ago, minutes / 60)
        else -> stringResource(Res.string.time_days_ago, minutes / (24 * 60))
    }
}
