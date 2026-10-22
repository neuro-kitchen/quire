package dev.neurokitchen.quire.presentation.screens.notifications

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.repository.NotificationsRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * This device's notifications (`.tasks/T14-security.md`): the unread count follows the library
 * (a sync adds entries); opening the list shows them and marks them read.
 */
class NotificationsViewModel(private val notifications: NotificationsRepository) : ViewModel() {
    private val _state = MutableStateFlow(NotificationsState())
    val state: StateFlow<NotificationsState> = _state.asStateFlow()

    init {
        viewModelScope.launch {
            notifications.unread().catch { emit(0) }.collect { unread ->
                _state.update { it.copy(unread = unread) }
                // Open: new entries show at once (and are read, being seen).
                if (_state.value.isOpen && unread > 0) load()
            }
        }
    }

    fun onEvent(event: NotificationsEvent) {
        when (event) {
            NotificationsEvent.OnOpen -> {
                _state.update { it.copy(isOpen = true) }
                viewModelScope.launch { load() }
            }
            NotificationsEvent.OnClose -> _state.update { it.copy(isOpen = false) }
        }
    }

    /** The list as it is, then everything marked read (the badge goes). */
    private suspend fun load() {
        val items = runCatching { notifications.list() }.getOrDefault(emptyList())
        _state.update { it.copy(items = items) }
        if (items.any { !it.read }) runCatching { notifications.markRead() }
    }
}
