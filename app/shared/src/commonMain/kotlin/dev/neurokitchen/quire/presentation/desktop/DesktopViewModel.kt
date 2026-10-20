package dev.neurokitchen.quire.presentation.desktop

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

class DesktopViewModel : ViewModel() {
    private val _state = MutableStateFlow(DesktopState())
    val state: StateFlow<DesktopState> = _state.asStateFlow()

    fun onEvent(event: DesktopEvent) {
        when (event) {
            is DesktopEvent.OnOpenPaper -> _state.update { state ->
                val papers = if (state.papers.any { it.id == event.id }) state.papers else state.papers + OpenPaper(event.id, event.title)
                state.copy(papers = papers, selected = event.id)
            }
            is DesktopEvent.OnSelectTab -> _state.update { it.copy(selected = event.id) }
            is DesktopEvent.OnCloseTab -> _state.update { state ->
                val index = state.papers.indexOfFirst { it.id == event.id }
                val papers = state.papers.filter { it.id != event.id }
                // Closing the open tab moves to its left neighbour (the library after the first).
                val selected = if (state.selected == event.id) papers.getOrNull(index - 1)?.id else state.selected
                state.copy(papers = papers, selected = selected)
            }
        }
    }
}
