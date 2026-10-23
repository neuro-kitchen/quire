package dev.neurokitchen.quire.presentation.screens.notifications

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.ActivityItem

@Immutable
data class NotificationsState(
    /** Unread notifications (the bell's badge). */
    val unread: Int = 0,
    /** The list, newest first; `null` until opened. */
    val items: List<ActivityItem>? = null,
    val isOpen: Boolean = false,
)
