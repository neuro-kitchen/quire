package dev.neurokitchen.quire.presentation.screens.notifications.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.domain.model.ActivityItem
import dev.neurokitchen.quire.domain.model.ActivityKind
import dev.neurokitchen.quire.presentation.components.time.ago
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_sync
import dev.neurokitchen.quire.resources.ic_warning
import dev.neurokitchen.quire.resources.notification_count_attachment
import dev.neurokitchen.quire.resources.notification_count_highlight
import dev.neurokitchen.quire.resources.notification_count_note
import dev.neurokitchen.quire.resources.notification_count_other
import dev.neurokitchen.quire.resources.notification_count_project
import dev.neurokitchen.quire.resources.notification_count_reading
import dev.neurokitchen.quire.resources.notification_count_reference
import dev.neurokitchen.quire.resources.notification_count_tag
import dev.neurokitchen.quire.resources.notification_key_changed
import dev.neurokitchen.quire.resources.notification_link_allowed
import dev.neurokitchen.quire.resources.notification_link_refused
import dev.neurokitchen.quire.resources.notification_linked
import dev.neurokitchen.quire.resources.notification_new_device
import dev.neurokitchen.quire.resources.notification_none
import dev.neurokitchen.quire.resources.notification_sync
import dev.neurokitchen.quire.resources.notification_unlinked
import dev.neurokitchen.quire.resources.notification_unnamed
import dev.neurokitchen.quire.resources.notification_wrong_code
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

/** The notifications, newest first (desktop panel and phone page alike). */
@Composable
fun NotificationsList(items: List<ActivityItem>, modifier: Modifier = Modifier) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm)) {
        if (items.isEmpty()) {
            Text(stringResource(Res.string.notification_none), style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary)
        }
        items.forEach { NotificationRow(it) }
    }
}

@Composable
private fun NotificationRow(item: ActivityItem) {
    val colors = QuireTheme.colors
    val icon = when {
        item.kind.isWarning -> Res.drawable.ic_warning
        item.kind == ActivityKind.Sync -> Res.drawable.ic_sync
        else -> Res.drawable.ic_link
    }
    Row(verticalAlignment = Alignment.Top, modifier = Modifier.fillMaxWidth()) {
        Icon(
            painterResource(icon),
            contentDescription = null,
            tint = if (item.kind.isWarning) colors.accentRed else colors.accent,
            modifier = Modifier.padding(top = 2.dp, end = Dimens.SpaceXs).size(16.dp),
        )
        Column(Modifier.weight(1f)) {
            Text(
                sentence(item),
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = if (item.read) FontWeight.Normal else FontWeight.SemiBold,
                color = colors.fillPrimary,
            )
            Text(ago(item.at), style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
        }
    }
}

/** The notification in words: what came from which device, or what happened. */
@Composable
private fun sentence(item: ActivityItem): String {
    val source = item.source ?: stringResource(Res.string.notification_unnamed)
    return when (item.kind) {
        ActivityKind.Sync -> stringResource(Res.string.notification_sync, source, counts(item.counts))
        ActivityKind.Linked -> stringResource(Res.string.notification_linked)
        ActivityKind.LinkAllowed -> stringResource(Res.string.notification_link_allowed, source)
        ActivityKind.LinkRefused -> stringResource(Res.string.notification_link_refused, source)
        ActivityKind.WrongCode -> pluralStringResource(Res.plurals.notification_wrong_code, item.number ?: 1, item.number ?: 1)
        ActivityKind.Unlinked -> stringResource(Res.string.notification_unlinked, source)
        ActivityKind.KeyChanged -> stringResource(Res.string.notification_key_changed)
        ActivityKind.NewDevice -> stringResource(Res.string.notification_new_device, source)
        ActivityKind.Other -> source
    }
}

/** "2 notes, 1 highlight, 1 paper's reading status": the kinds a person knows; the rest counted. */
@Composable
private fun counts(counts: List<Pair<String, Int>>): String {
    val known = mapOf(
        "reference" to Res.plurals.notification_count_reference,
        "note" to Res.plurals.notification_count_note,
        "highlight" to Res.plurals.notification_count_highlight,
        "reading" to Res.plurals.notification_count_reading,
        "project" to Res.plurals.notification_count_project,
        "tag" to Res.plurals.notification_count_tag,
        "attachment" to Res.plurals.notification_count_attachment,
    )
    val parts = counts.mapNotNull { (kind, n) -> known[kind]?.let { pluralStringResource(it, n, n) } }
    val other = counts.filter { it.first !in known }.sumOf { it.second }
    return (parts + listOfNotNull(if (other > 0) pluralStringResource(Res.plurals.notification_count_other, other, other) else null)).joinToString(", ")
}
