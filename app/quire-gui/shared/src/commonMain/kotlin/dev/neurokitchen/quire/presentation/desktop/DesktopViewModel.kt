package dev.neurokitchen.quire.presentation.desktop

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.data.repository.SamplePreferencesRepository
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

@OptIn(ExperimentalCoroutinesApi::class)
class DesktopViewModel(
    private val preferencesRepository: PreferencesRepository = SamplePreferencesRepository(),
    private val libraryRepository: LibraryRepository = SampleLibraryRepository(),
) : ViewModel() {
    private val _state = MutableStateFlow(DesktopState(appTheme = preferencesRepository.preferences.value.appTheme))
    val state: StateFlow<DesktopState> = _state.asStateFlow()

    init {
        viewModelScope.launch {
            preferencesRepository.preferences.collect { prefs ->
                _state.update { it.copy(appTheme = prefs.appTheme) }
            }
        }
        // After an update, papers' stored texts are rebuilt in the background, a moment after the
        // window is up, instead of each one inside its first opening.
        viewModelScope.launch {
            delay(STALE_TEXTS_DELAY_MS)
            runCatching { libraryRepository.rebuildStaleTexts() }
        }
        // Every reference, for the command palette and the citation graph: followed only while one
        // of them is open (the library screen keeps its own list).
        viewModelScope.launch {
            _state.map { it.isCommandPaletteOpen || it.isCitationGraphOpen }
                .distinctUntilChanged()
                .flatMapLatest { open -> if (open) libraryRepository.references(LibraryQuery()).catch { } else emptyFlow() }
                .collect { refs -> _state.update { it.copy(allReferences = refs) } }
        }
    }

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
            is DesktopEvent.OnSetAppTheme -> {
                _state.update { it.copy(appTheme = event.theme) }
                viewModelScope.launch { preferencesRepository.setAppTheme(event.theme) }
            }
            is DesktopEvent.OnToggleCommandPalette -> _state.update { it.copy(isCommandPaletteOpen = event.open) }
            is DesktopEvent.OnToggleCitationGraph -> _state.update { it.copy(isCitationGraphOpen = event.open) }
        }
    }
}

/** How long after startup stale paper texts start rebuilding (the first screen goes first). */
private const val STALE_TEXTS_DELAY_MS = 3_000L
