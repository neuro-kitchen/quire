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
            val selected = references.firstOrNull { it.id == view.selectedId }
            LibraryState(
                isLoading = false,
                query = query.text,
                source = query.source,
                projects = projectRows(projects, query.source, view.expandedProjects),
                tags = (references.flatMap { it.tags } + query.tags).distinct().sorted()
                    .map { TagState(it, isSelected = it in query.tags) },
                sort = query.sort,
                ascending = query.ascending,
                references = references.map { ReferenceCardState.from(it, isSelected = it.id == view.selectedId) },
                selected = selected?.let { detail(it, projects) },
                collapsedSections = view.collapsedSections,
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
            is LibraryEvent.OnReferenceClick -> view.update { it.copy(selectedId = event.id) }
            LibraryEvent.OnCloseDetail -> view.update { it.copy(selectedId = null) }
            is LibraryEvent.OnSectionToggle ->
                view.update { it.copy(collapsedSections = it.collapsedSections.toggle(event.section)) }
        }
    }

    /** What the screen shows that is not part of the query. */
    private data class ViewOptions(
        val selectedId: String? = null,
        val expandedProjects: Set<String> = emptySet(),
        val collapsedSections: Set<ItemSection> = emptySet(),
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

