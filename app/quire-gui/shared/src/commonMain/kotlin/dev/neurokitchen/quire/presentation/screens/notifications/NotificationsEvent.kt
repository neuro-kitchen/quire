package dev.neurokitchen.quire.presentation.screens.notifications

sealed interface NotificationsEvent {
    /** Shows the list (and marks everything read). */
    data object OnOpen : NotificationsEvent
    data object OnClose : NotificationsEvent
}
