package dev.neurokitchen.quire.presentation.mobile.screens.library

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.Project
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import dev.neurokitchen.quire.presentation.screens.library.ProjectRowState
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

@OptIn(ExperimentalCoroutinesApi::class)
class MobileLibraryViewModel(
    private val library: LibraryRepository,
) : ViewModel() {

    private val source = MutableStateFlow<LibrarySource>(LibrarySource.All)
    private val isFolderSheetVisible = MutableStateFlow(false)
    private val isCreateProjectDialogOpen = MutableStateFlow(false)
    private val expandedProjects = MutableStateFlow<Set<String>>(emptySet())
    private val actionSheetRef = MutableStateFlow<ReferenceCardState?>(null)
    private val confirmDeleteRef = MutableStateFlow<ReferenceCardState?>(null)
    private val isRefreshing = MutableStateFlow(false)

    private val referencesFlow = source.flatMapLatest { currentSource ->
        library.references(LibraryQuery(source = currentSource))
            .map { refs -> Result.success(refs.map { ReferenceCardState.from(it) }) }
            .catch { emit(Result.failure(it)) }
    }

    private val baseState: Flow<MobileLibraryUiState> = combine(
        source,
        isFolderSheetVisible,
        expandedProjects,
        library.projects(),
        referencesFlow,
    ) { currentSource, isSheetVisible, expanded, projects, refResult ->
        val failure = refResult.exceptionOrNull()
        val cards = refResult.getOrDefault(emptyList())
        MobileLibraryUiState(
            source = currentSource,
            references = cards,
            projects = projectRows(projects, currentSource, expanded),
            projectChoices = projects.map { it.path to it.name },
            isFolderSheetVisible = isSheetVisible,
            isLoading = false,
            error = failure?.message ?: failure?.toString(),
        )
    }

    val state: StateFlow<MobileLibraryUiState> = combine(
        baseState,
        actionSheetRef,
        confirmDeleteRef,
        isCreateProjectDialogOpen,
        isRefreshing,
    ) { base, sheetRef, deleteRef, isCreateOpen, refreshing ->
        base.copy(
            actionSheetReference = sheetRef,
            confirmDeleteReference = deleteRef,
            isCreateProjectDialogOpen = isCreateOpen,
            isRefreshing = refreshing,
        )
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = MobileLibraryUiState(isLoading = true),
    )

    fun onEvent(event: MobileLibraryEvent) {
        when (event) {
            is MobileLibraryEvent.OnSelectSource -> {
                source.value = event.source
                isFolderSheetVisible.value = false
            }
            is MobileLibraryEvent.SetFolderSheetVisible -> {
                isFolderSheetVisible.value = event.visible
            }
            MobileLibraryEvent.OnOpenCreateProjectDialog -> {
                isFolderSheetVisible.value = false
                isCreateProjectDialogOpen.value = true
            }
            MobileLibraryEvent.OnDismissCreateProjectDialog -> {
                isCreateProjectDialogOpen.value = false
            }
            is MobileLibraryEvent.OnCreateProject -> {
                isCreateProjectDialogOpen.value = false
                viewModelScope.launch {
                    val name = event.name.trim()
                    if (name.isNotEmpty()) {
                        val path = library.createProject(name)
                        source.value = LibrarySource.InProject(path)
                    }
                }
            }
            is MobileLibraryEvent.OnProjectExpandToggle -> {
                expandedProjects.update { set ->
                    if (event.projectId in set) set - event.projectId else set + event.projectId
                }
            }
            is MobileLibraryEvent.OnReferenceLongClick -> actionSheetRef.value = event.card
            MobileLibraryEvent.OnDismissActionSheet -> actionSheetRef.value = null
            is MobileLibraryEvent.OnRequestDelete -> {
                actionSheetRef.value = null
                confirmDeleteRef.value = event.card
            }
            MobileLibraryEvent.OnDismissDeleteDialog -> confirmDeleteRef.value = null
            MobileLibraryEvent.OnConfirmDelete -> {
                val ref = confirmDeleteRef.value
                confirmDeleteRef.value = null
                if (ref != null) {
                    viewModelScope.launch {
                        library.deleteReference(ref.id)
                    }
                }
            }
            is MobileLibraryEvent.OnDownloadPdf -> {
                actionSheetRef.value = null
                viewModelScope.launch {
                    val all = library.references(LibraryQuery()).first()
                    val item = all.find { it.id == event.id }
                    if (item != null) {
                        library.updateReference(item.copy(hasPdf = true))
                    }
                }
            }
            MobileLibraryEvent.OnRefresh -> {
                viewModelScope.launch {
                    isRefreshing.value = true
                    try {
                        library.reopen()
                        library.rebuildStaleTexts()
                    } catch (_: Exception) {
                    } finally {
                        isRefreshing.value = false
                    }
                }
            }
        }
    }

    private fun projectRows(projects: List<Project>, source: LibrarySource, expanded: Set<String>): List<ProjectRowState> {
        val children = projects.groupBy { it.parentPath }
        val rows = mutableListOf<ProjectRowState>()
        fun add(parent: String?, depth: Int) {
            for (project in children[parent].orEmpty().sortedBy { it.name.lowercase() }) {
                val hasChildren = children[project.path].orEmpty().isNotEmpty()
                val isExpanded = project.path in expanded
                rows += ProjectRowState(
                    id = project.path,
                    name = project.name,
                    depth = depth,
                    hasChildren = hasChildren,
                    isExpanded = isExpanded,
                    isSelected = source == LibrarySource.InProject(project.path),
                    referenceCount = project.referenceCount,
                )
                if (hasChildren && isExpanded) add(project.path, depth + 1)
            }
        }
        add(null, 0)
        return rows
    }
}
