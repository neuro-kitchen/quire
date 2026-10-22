package dev.neurokitchen.quire.presentation.mobile.screens.home

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState

@Immutable
data class HomeUiState(
    val daysCount: Int = 0,
    val streakCount: Int = 0,
    val activeDaysOfWeek: Set<Int> = emptySet(),
    val reading: List<ReferenceCardState> = emptyList(),
    val recent: List<ReferenceCardState> = emptyList(),
    val totalCount: Int = 0,
    val isLoading: Boolean = true,
    val error: String? = null,
    val actionSheetReference: ReferenceCardState? = null,
    val confirmDeleteReference: ReferenceCardState? = null,
)

sealed interface HomeEvent {
    data object Refresh : HomeEvent
    data class OnReferenceLongClick(val card: ReferenceCardState) : HomeEvent
    data object OnDismissActionSheet : HomeEvent
    data class OnRequestDelete(val card: ReferenceCardState) : HomeEvent
    data object OnDismissDeleteDialog : HomeEvent
    data object OnConfirmDelete : HomeEvent
    data class OnDownloadPdf(val id: String) : HomeEvent
}
