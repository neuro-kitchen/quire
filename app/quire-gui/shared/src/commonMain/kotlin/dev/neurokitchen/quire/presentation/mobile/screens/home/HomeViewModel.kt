package dev.neurokitchen.quire.presentation.mobile.screens.home

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

class HomeViewModel(
    private val library: LibraryRepository,
) : ViewModel() {

    private val actionSheetRef = MutableStateFlow<ReferenceCardState?>(null)
    private val confirmDeleteRef = MutableStateFlow<ReferenceCardState?>(null)

    val state: StateFlow<HomeUiState> = combine(
        library.references(LibraryQuery()),
        actionSheetRef,
        confirmDeleteRef,
    ) { references, sheetRef, deleteRef ->
        val cards = references.map { ReferenceCardState.from(it) }
        val reading = cards.filter { it.reading == ReadingStatus.Reading || it.reading == ReadingStatus.Read }
        val recent = cards.sortedByDescending { it.dateAdded }
        val daysCount = maxOf(reading.size, 5)
        HomeUiState(
            daysCount = daysCount,
            streakCount = 5,
            activeDaysOfWeek = setOf(0, 1, 2, 3, 4),
            reading = reading,
            recent = recent,
            totalCount = cards.size,
            isLoading = false,
            error = null,
            actionSheetReference = sheetRef,
            confirmDeleteReference = deleteRef,
        )
    }.catch { error ->
        emit(
            HomeUiState(
                isLoading = false,
                error = error.message ?: error.toString(),
            ),
        )
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = HomeUiState(isLoading = true),
    )

    fun onEvent(event: HomeEvent) {
        when (event) {
            HomeEvent.Refresh -> Unit
            is HomeEvent.OnReferenceLongClick -> actionSheetRef.value = event.card
            HomeEvent.OnDismissActionSheet -> actionSheetRef.value = null
            is HomeEvent.OnRequestDelete -> {
                actionSheetRef.value = null
                confirmDeleteRef.value = event.card
            }
            HomeEvent.OnDismissDeleteDialog -> confirmDeleteRef.value = null
            HomeEvent.OnConfirmDelete -> {
                val ref = confirmDeleteRef.value
                confirmDeleteRef.value = null
                if (ref != null) {
                    viewModelScope.launch {
                        library.deleteReference(ref.id)
                    }
                }
            }
            is HomeEvent.OnDownloadPdf -> {
                actionSheetRef.value = null
                viewModelScope.launch {
                    val all = library.references(LibraryQuery()).first()
                    val item = all.find { it.id == event.id }
                    if (item != null) {
                        library.updateReference(item.copy(hasPdf = true))
                    }
                }
            }
        }
    }
}
