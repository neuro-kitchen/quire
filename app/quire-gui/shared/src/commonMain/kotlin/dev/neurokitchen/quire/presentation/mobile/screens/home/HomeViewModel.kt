package dev.neurokitchen.quire.presentation.mobile.screens.home

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.data.repository.SamplePreferencesRepository
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.PreferencesRepository
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.datetime.DateTimeUnit
import kotlinx.datetime.LocalDate
import kotlinx.datetime.TimeZone
import kotlinx.datetime.isoDayNumber
import kotlinx.datetime.minus
import kotlinx.datetime.plus
import kotlinx.datetime.todayIn
import kotlin.time.Clock
import kotlin.time.ExperimentalTime

@OptIn(ExperimentalTime::class)
class HomeViewModel(
    private val library: LibraryRepository,
    private val preferences: PreferencesRepository = SamplePreferencesRepository(),
) : ViewModel() {

    private val actionSheetRef = MutableStateFlow<ReferenceCardState?>(null)
    private val confirmDeleteRef = MutableStateFlow<ReferenceCardState?>(null)

    val state: StateFlow<HomeUiState> = combine(
        library.references(LibraryQuery()),
        preferences.preferences,
        actionSheetRef,
        confirmDeleteRef,
    ) { references, prefs, sheetRef, deleteRef ->
        val cards = references.map { ReferenceCardState.from(it) }
        val reading = cards.filter { it.reading == ReadingStatus.Reading || it.reading == ReadingStatus.Read }
        val recent = cards.sortedByDescending { it.dateAdded }

        // Common code runs on every platform (iOS too): kotlinx-datetime, not java.time.
        val today = runCatching { Clock.System.todayIn(TimeZone.currentSystemDefault()) }.getOrNull()

        val readingDates = prefs.readingDays.mapNotNull { runCatching { LocalDate.parse(it) }.getOrNull() }.toMutableSet()

        if (today != null && reading.isNotEmpty() && readingDates.isEmpty()) {
            readingDates.add(today)
            viewModelScope.launch { preferences.recordReadingToday() }
        }

        val activeDaysOfWeek = mutableSetOf<Int>()
        var streakCount = 0

        if (today != null) {
            val dayOfWeekVal = today.dayOfWeek.isoDayNumber // 1 (Mon) .. 7 (Sun)
            val monday = today.minus(dayOfWeekVal - 1, DateTimeUnit.DAY)
            for (i in 0..6) {
                val d = monday.plus(i, DateTimeUnit.DAY)
                if (d in readingDates) {
                    activeDaysOfWeek.add(i)
                }
            }

            var checkDate = if (today in readingDates) today else today.minus(1, DateTimeUnit.DAY)
            while (checkDate in readingDates) {
                streakCount++
                checkDate = checkDate.minus(1, DateTimeUnit.DAY)
            }
        }

        val daysCount = maxOf(readingDates.size, if (reading.isNotEmpty()) 1 else 0)

        HomeUiState(
            daysCount = daysCount,
            streakCount = streakCount,
            activeDaysOfWeek = activeDaysOfWeek,
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
