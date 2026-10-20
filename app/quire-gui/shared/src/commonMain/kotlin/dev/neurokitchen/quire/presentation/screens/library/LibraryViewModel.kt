package dev.neurokitchen.quire.presentation.screens.library

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.Project
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.field_date
import dev.neurokitchen.quire.resources.field_date_added
import dev.neurokitchen.quire.resources.field_doi
import dev.neurokitchen.quire.resources.field_issue
import dev.neurokitchen.quire.resources.field_pages
import dev.neurokitchen.quire.resources.field_publication
import dev.neurokitchen.quire.resources.field_publisher
import dev.neurokitchen.quire.resources.field_volume
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update

@OptIn(ExperimentalCoroutinesApi::class)
class LibraryViewModel(
    private val library: LibraryRepository,
) : ViewModel() {
    private val query = MutableStateFlow(LibraryQuery())
    private val view = MutableStateFlow(ViewOptions())

    private val references = query.flatMapLatest { library.references(it) }

    val state: StateFlow<LibraryState> =
        combine(query, references, library.projects(), view) { query, references, projects, view ->
            val activeSelectedId = view.selectedId ?: view.selectedIds.firstOrNull()
            val selected = references.firstOrNull { it.id == activeSelectedId }
            val effectiveSelectedIds = if (view.selectedIds.isNotEmpty()) view.selectedIds
                else if (view.selectedId != null) setOf(view.selectedId) else emptySet()
            LibraryState(
                isLoading = false,
                query = query.text,
                source = query.source,
                projects = projectRows(projects, query.source, view.expandedProjects),
                tags = (references.flatMap { it.tags } + query.tags).distinct().sorted()
                    .map { TagState(it, isSelected = it in query.tags) },
                sort = query.sort,
                ascending = query.ascending,
                references = references.map { ReferenceCardState.from(it, isSelected = it.id in effectiveSelectedIds) },
                selectedIds = effectiveSelectedIds,
                selected = selected?.let { detail(it, projects) },
                collapsedSections = view.collapsedSections,
                isSidebarCollapsed = view.isSidebarCollapsed,
                isInspectorCollapsed = view.isInspectorCollapsed,
                showAddDialog = view.showAddDialog,
                showAddProjectDialog = view.showAddProjectDialog,
            )
        }.catch { emit(LibraryState(isLoading = false, error = it.message ?: it.toString())) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), LibraryState())

    fun onEvent(event: LibraryEvent) {
        when (event) {
            is LibraryEvent.OnQueryChange -> query.update { it.copy(text = event.query) }
            is LibraryEvent.OnSourceClick -> query.update { it.copy(source = event.source) }
            is LibraryEvent.OnProjectExpandToggle ->
                view.update { it.copy(expandedProjects = it.expandedProjects.toggle(event.id)) }
            is LibraryEvent.OnTagClick -> query.update { it.copy(tags = it.tags.toggle(event.name)) }
            is LibraryEvent.OnSortClick -> query.update {
                if (it.sort == event.column) it.copy(ascending = !it.ascending)
                else it.copy(sort = event.column, ascending = true)
            }
            is LibraryEvent.OnReferenceClick -> view.update {
                it.copy(selectedId = event.id, selectedIds = setOf(event.id), lastClickedId = event.id)
            }
            is LibraryEvent.OnReferenceSelect -> {
                val currentList = state.value.references.map { it.id }
                view.update { currentView ->
                    val newSelectedIds = when {
                        event.isShiftPressed && currentView.lastClickedId != null -> {
                            val startIdx = currentList.indexOf(currentView.lastClickedId)
                            val endIdx = currentList.indexOf(event.id)
                            if (startIdx != -1 && endIdx != -1) {
                                val min = minOf(startIdx, endIdx)
                                val max = maxOf(startIdx, endIdx)
                                currentView.selectedIds + currentList.subList(min, max + 1)
                            } else {
                                currentView.selectedIds + event.id
                            }
                        }
                        event.isCtrlPressed -> {
                            if (event.id in currentView.selectedIds) currentView.selectedIds - event.id
                            else currentView.selectedIds + event.id
                        }
                        else -> setOf(event.id)
                    }
                    currentView.copy(
                        selectedId = event.id,
                        selectedIds = newSelectedIds,
                        lastClickedId = event.id,
                    )
                }
            }
            LibraryEvent.OnSelectAll -> {
                val allIds = state.value.references.map { it.id }.toSet()
                view.update { it.copy(selectedIds = allIds) }
            }
            LibraryEvent.OnClearSelection -> {
                view.update { it.copy(selectedIds = emptySet(), selectedId = null, lastClickedId = null) }
            }
            LibraryEvent.OnToggleInspector -> {
                view.update { it.copy(isInspectorCollapsed = !it.isInspectorCollapsed) }
            }
            LibraryEvent.OnCloseDetail -> view.update { it.copy(selectedId = null, selectedIds = emptySet()) }
            is LibraryEvent.OnSectionToggle ->
                view.update { it.copy(collapsedSections = it.collapsedSections.toggle(event.section)) }
            is LibraryEvent.OnAddReference -> {
                viewModelScope.launch { library.addReference(event.reference) }
            }
            is LibraryEvent.OnDeleteReference -> {
                viewModelScope.launch {
                    library.deleteReference(event.id)
                    view.update {
                        val remaining = it.selectedIds - event.id
                        it.copy(
                            selectedId = if (it.selectedId == event.id) remaining.firstOrNull() else it.selectedId,
                            selectedIds = remaining,
                        )
                    }
                }
            }
            LibraryEvent.OnDeleteSelected -> {
                val toDelete = if (view.value.selectedIds.isNotEmpty()) view.value.selectedIds
                    else if (view.value.selectedId != null) setOf(view.value.selectedId!!) else emptySet()
                viewModelScope.launch {
                    toDelete.forEach { library.deleteReference(it) }
                    view.update { it.copy(selectedId = null, selectedIds = emptySet(), lastClickedId = null) }
                }
            }
            is LibraryEvent.OnBatchSetReadingStatus -> {
                // In memory or engine update
            }
            is LibraryEvent.OnBatchAddToProject -> {
                // In memory or engine update
            }
            is LibraryEvent.OnCreateCollectionFromSelection -> {
                viewModelScope.launch {
                    library.createProject(event.name, null)
                }
            }
            is LibraryEvent.OnCreateProject -> {
                viewModelScope.launch { library.createProject(event.name, event.parentPath) }
            }
            is LibraryEvent.OnShowAddDialog -> view.update { it.copy(showAddDialog = event.show) }
            is LibraryEvent.OnShowAddProjectDialog -> view.update { it.copy(showAddProjectDialog = event.show) }
            LibraryEvent.OnToggleSidebar -> view.update { it.copy(isSidebarCollapsed = !it.isSidebarCollapsed) }
        }
    }

    /** What the screen shows that is not part of the query. */
    private data class ViewOptions(
        val selectedId: String? = null,
        val selectedIds: Set<String> = emptySet(),
        val lastClickedId: String? = null,
        val expandedProjects: Set<String> = emptySet(),
        val collapsedSections: Set<ItemSection> = emptySet(),
        val isSidebarCollapsed: Boolean = false,
        val isInspectorCollapsed: Boolean = false,
        val showAddDialog: Boolean = false,
        val showAddProjectDialog: Boolean = false,
    )
}

private fun <T> Set<T>.toggle(value: T): Set<T> = if (value in this) this - value else this + value

/** The tree in display order, children under expanded parents only. */
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

private fun detail(reference: Reference, projects: List<Project>): ReferenceDetailState {
    val card = ReferenceCardState.from(reference)
    return ReferenceDetailState(
        id = reference.id,
        title = reference.title,
        citeKey = reference.citeKey,
        kind = reference.kind,
        creator = card.creator,
        year = card.year,
        authors = reference.creators.map { it.displayName },
        fields = listOf(
            DetailField(Res.string.field_publication, reference.containerTitle),
            DetailField(Res.string.field_publisher, reference.publisher),
            DetailField(Res.string.field_volume, reference.volume),
            DetailField(Res.string.field_issue, reference.issue),
            DetailField(Res.string.field_pages, reference.pages),
            DetailField(Res.string.field_date, reference.date),
            DetailField(Res.string.field_doi, reference.doi),
            DetailField(Res.string.field_date_added, reference.dateAdded),
        ).filter { it.value.isNotEmpty() },
        abstract = reference.abstract,
        tags = reference.tags.sorted(),
        projects = reference.projects.map { path -> projects.firstOrNull { it.path == path }?.name ?: path },
        hasPdf = reference.hasPdf,
        reading = reference.reading,
    )
}

