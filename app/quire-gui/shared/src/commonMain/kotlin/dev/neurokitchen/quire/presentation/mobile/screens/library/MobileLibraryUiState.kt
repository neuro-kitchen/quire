package dev.neurokitchen.quire.presentation.mobile.screens.library

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import dev.neurokitchen.quire.presentation.screens.library.ProjectRowState

@Immutable
data class MobileLibraryUiState(
    val source: LibrarySource = LibrarySource.All,
    val references: List<ReferenceCardState> = emptyList(),
    val projects: List<ProjectRowState> = emptyList(),
    val projectChoices: List<Pair<String, String>> = emptyList(),
    val isFolderSheetVisible: Boolean = false,
    val isCreateProjectDialogOpen: Boolean = false,
    val isLoading: Boolean = true,
    val error: String? = null,
    val actionSheetReference: ReferenceCardState? = null,
    val confirmDeleteReference: ReferenceCardState? = null,
)

sealed interface MobileLibraryEvent {
    data class OnSelectSource(val source: LibrarySource) : MobileLibraryEvent
    data class SetFolderSheetVisible(val visible: Boolean) : MobileLibraryEvent
    data object OnOpenCreateProjectDialog : MobileLibraryEvent
    data object OnDismissCreateProjectDialog : MobileLibraryEvent
    data class OnCreateProject(val name: String) : MobileLibraryEvent
    data class OnProjectExpandToggle(val projectId: String) : MobileLibraryEvent
    data class OnReferenceLongClick(val card: ReferenceCardState) : MobileLibraryEvent
    data object OnDismissActionSheet : MobileLibraryEvent
    data class OnRequestDelete(val card: ReferenceCardState) : MobileLibraryEvent
    data object OnDismissDeleteDialog : MobileLibraryEvent
    data object OnConfirmDelete : MobileLibraryEvent
    data class OnDownloadPdf(val id: String) : MobileLibraryEvent
}
