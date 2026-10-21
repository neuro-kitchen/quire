package dev.neurokitchen.quire.presentation.screens.library

import dev.neurokitchen.quire.presentation.components.notice.Notice
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.domain.model.AddOutcome
import dev.neurokitchen.quire.domain.model.Creator
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.Project
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.SavedSearch
import dev.neurokitchen.quire.domain.model.Tag
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.domain.model.PdfVerdict
import dev.neurokitchen.quire.domain.repository.AttachmentRepository
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import dev.neurokitchen.quire.domain.repository.SearchRepository
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
import dev.neurokitchen.quire.resources.notice_added
import dev.neurokitchen.quire.resources.tools_zotero_imported
import dev.neurokitchen.quire.resources.notice_added_many
import dev.neurokitchen.quire.resources.notice_added_to_project
import dev.neurokitchen.quire.resources.notice_already_in_library
import dev.neurokitchen.quire.resources.notice_imported
import dev.neurokitchen.quire.resources.notice_project_created
import dev.neurokitchen.quire.resources.notice_project_deleted
import dev.neurokitchen.quire.resources.notice_project_renamed
import dev.neurokitchen.quire.resources.notice_tag_deleted
import dev.neurokitchen.quire.resources.notice_tag_renamed
import dev.neurokitchen.quire.resources.notice_search_removed
import dev.neurokitchen.quire.resources.notice_search_saved
import dev.neurokitchen.quire.resources.notice_reading_set
import dev.neurokitchen.quire.resources.notice_trashed
import dev.neurokitchen.quire.resources.notice_saved
import dev.neurokitchen.quire.resources.notice_note_removed
import dev.neurokitchen.quire.resources.notice_pdf_downloaded
import dev.neurokitchen.quire.resources.notice_pdf_present
import dev.neurokitchen.quire.resources.notice_pdf_attached
import dev.neurokitchen.quire.resources.notice_pdf_other_paper
import dev.neurokitchen.quire.resources.notice_pdfs_removed
import dev.neurokitchen.quire.resources.tools_exported
import dev.neurokitchen.quire.resources.notice_no_publisher_page
import dev.neurokitchen.quire.resources.notice_nothing_changed
import dev.neurokitchen.quire.resources.notice_refetched
import dev.neurokitchen.quire.resources.notice_restored
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update

@OptIn(ExperimentalCoroutinesApi::class)
class LibraryViewModel(
    private val library: LibraryRepository,
    private val reader: ReaderRepository,
    private val exchange: ExchangeRepository,
    private val attachments: AttachmentRepository,
    private val search: SearchRepository,
) : ViewModel() {
    private val query = MutableStateFlow(LibraryQuery())
    private val view = MutableStateFlow(ViewOptions())

    /** The references for a query, or why the engine refused it (a search filter half typed), with the query. */
    private val results = query.flatMapLatest { q ->
        library.references(q).map { q to Result.success(it) }.catch { emit(q to Result.failure(it)) }
    }

    /** The references in view, as last shown (to edit one from its full record). */
    private var shown: List<Reference> = emptyList()

    /**
     * The selected reference's notes, related references, PDFs and issues; loaded again when the
     * selection or the library changes. Each part on its own: one that fails (no engine) is empty.
     */
    private val extras = combine(view.map { it.selectedId ?: it.selectedIds.firstOrNull() }.distinctUntilChanged(), results) { id, _ -> id }
        .flatMapLatest { id ->
            if (id == null) {
                flowOf(Extras())
            } else {
                reader.notes(id).catch { emit(emptyList()) }.mapLatest { notes ->
                    Extras(
                        id = id,
                        notes = notes,
                        related = runCatching { library.related(id) }.getOrDefault(emptyList()),
                        pdfFiles = runCatching { reader.pdfFiles(id) }.getOrDefault(emptyList()),
                        issues = runCatching { library.issues(id).map { it.message } }.getOrDefault(emptyList()),
                    )
                }
            }
        }

    /** The library's tags with their colours; none without the engine. */
    private val libraryTags = flow { emitAll(library.tags()) }.catch { emit(emptyList()) }

    private val savedSearches = MutableStateFlow<List<SavedSearch>>(emptyList())

    private val projectsAndTags = combine(library.projects(), libraryTags, savedSearches, ::Triple)

    init {
        reloadSavedSearches()
    }

    /** Saved searches live in the engine's config, not the library: read again after each change. */
    private fun reloadSavedSearches() {
        viewModelScope.launch { savedSearches.value = runCatching { search.savedSearches() }.getOrDefault(emptyList()) }
    }

    val state: StateFlow<LibraryState> =
        combine(query, results, projectsAndTags, view, extras) { query, (answered, result), (projects, libraryTags, saved), view, extras ->
            val failure = result.exceptionOrNull()
            // Without a search, a failure is the library's; with one, the search's (keep what was shown).
            if (failure != null && answered.text.isBlank()) {
                return@combine LibraryState(isLoading = false, error = failure.message ?: failure.toString())
            }
            val references = result.getOrElse { shown }
            shown = references
            val activeSelectedId = view.selectedId ?: view.selectedIds.firstOrNull()
            val selected = references.firstOrNull { it.id == activeSelectedId }
            val effectiveSelectedIds = if (view.selectedIds.isNotEmpty()) view.selectedIds
                else if (view.selectedId != null) setOf(view.selectedId) else emptySet()
            LibraryState(
                isLoading = false,
                query = query.text,
                source = query.source,
                projects = projectRows(projects, query.source, view.expandedProjects),
                projectChoices = projects.map { it.path to it.name },
                tags = tagRows(libraryTags, references, query.tags),
                sort = query.sort,
                ascending = query.ascending,
                references = references.map { ReferenceCardState.from(it, isSelected = it.id in effectiveSelectedIds) },
                selectedIds = effectiveSelectedIds,
                selected = selected?.let {
                    val e = extras.takeIf { x -> x.id == it.id } ?: Extras()
                    detail(it, projects, e.notes, e.related, e.pdfFiles, e.issues, query.source == LibrarySource.Trash)
                },
                collapsedSections = view.collapsedSections,
                editing = view.editing,
                tagDraft = view.tagDraft,
                noteDraft = view.noteDraft,
                pageToOpen = view.pageToOpen,
                isSidebarCollapsed = view.isSidebarCollapsed,
                isInspectorCollapsed = view.isInspectorCollapsed,
                addDialog = view.addDialog,
                nameDialog = view.nameDialog,
                confirm = view.confirm,
                notice = view.notice,
                searchError = failure?.let { it.message ?: it.toString() },
                savedSearches = saved.map { SavedSearchState(it.name, it.query, isActive = it.query == query.text) },
                isQuickLookOpen = view.isQuickLookOpen,
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
            is LibraryEvent.OnDeleteReference -> act {
                library.deleteReference(event.id)
                view.update {
                    val remaining = it.selectedIds - event.id
                    it.copy(
                        selectedId = if (it.selectedId == event.id) remaining.firstOrNull() else it.selectedId,
                        selectedIds = remaining,
                    )
                }
                Notice.Done(Res.string.notice_trashed, listOf(1))
            }
            LibraryEvent.OnDeleteSelected -> {
                val toDelete = selection()
                act {
                    toDelete.forEach { library.deleteReference(it) }
                    view.update { it.copy(selectedId = null, selectedIds = emptySet(), lastClickedId = null) }
                    Notice.Done(Res.string.notice_trashed, listOf(toDelete.size))
                }
            }
            is LibraryEvent.OnBatchSetReadingStatus -> act {
                event.ids.forEach { reader.setReading(it, event.status) }
                Notice.Done(Res.string.notice_reading_set, listOf(event.ids.size))
            }
            is LibraryEvent.OnBatchAddToProject -> act {
                event.ids.forEach { library.addToProject(it, event.projectPath) }
                Notice.Done(Res.string.notice_added_to_project, listOf(event.ids.size, event.projectPath))
            }
            is LibraryEvent.OnCreateCollectionFromSelection -> act {
                val path = library.createProject(event.name, null)
                event.ids.forEach { library.addToProject(it, path) }
                Notice.Done(Res.string.notice_added_to_project, listOf(event.ids.size, event.name))
            }
            is LibraryEvent.OnCreateProject -> act {
                library.createProject(event.name, event.parentPath)
                Notice.Done(Res.string.notice_project_created, listOf(event.name))
            }
            is LibraryEvent.OnShowAddDialog -> view.update {
                it.copy(addDialog = if (event.show) AddDialogState(project = projectInView()) else null)
            }
            is LibraryEvent.OnAddPdfsPicked -> dialog { it.copy(pdfs = (it.pdfs + event.paths).distinct(), error = null) }
            is LibraryEvent.OnAddPdfRemove -> dialog { it.copy(pdfs = it.pdfs - event.path) }
            is LibraryEvent.OnAddFilePicked -> dialog { it.copy(file = event.path, error = null) }
            is LibraryEvent.OnAddProjectSelect -> dialog { it.copy(project = event.projectPath) }
            is LibraryEvent.OnAddTabSelect -> dialog { it.copy(tab = event.tab, error = null) }
            is LibraryEvent.OnIdentifierChange -> dialog { it.copy(identifier = event.identifier, preview = null, error = null) }
            is LibraryEvent.OnManualChange -> dialog { it.copy(manual = event.entry, error = null) }
            is LibraryEvent.OnBibtexChange -> dialog { it.copy(bibtex = event.text, error = null) }
            LibraryEvent.OnLookup -> working { dialog ->
                val preview = library.lookup(dialog.identifier.trim())
                dialog { it.copy(isWorking = false, preview = ReferenceCardState.from(preview)) }
            }
            LibraryEvent.OnAddConfirm -> working { dialog ->
                val project = dialog.project
                val notice = when (dialog.tab) {
                    AddTab.Identifier -> addIdentifiers(dialog, project) ?: return@working
                    AddTab.Pdf -> {
                        val outcomes = dialog.pdfs.map { attachments.importPdf(it, project).added }
                        outcomes.singleOrNull()?.let(::added)
                            ?: Notice.Done(Res.string.notice_added_many, listOf(outcomes.count { it.isNew }, outcomes.count { !it.isNew }))
                    }
                    AddTab.File -> {
                        val file = dialog.file ?: return@working
                        if (file.endsWith(".bib", ignoreCase = true)) {
                            Notice.Done(Res.string.notice_imported, listOf(exchange.importBib(file, project)))
                        } else {
                            val imported = exchange.importZotero(file, project)
                            Notice.Done(Res.string.tools_zotero_imported, listOf(imported.added, imported.existing))
                        }
                    }
                    AddTab.Manual -> {
                        val outcome = library.addReference(dialog.manual.toReference(project))
                        val tags = dialog.manual.tags.split(',').map { it.trim() }.filter { it.isNotEmpty() }
                        if (tags.isNotEmpty()) library.tag(outcome.reference.id, tags)
                        added(outcome)
                    }
                    AddTab.Bibtex -> Notice.Done(Res.string.notice_imported, listOf(exchange.importBibText(dialog.bibtex, project)))
                }
                view.update { it.copy(addDialog = null, notice = notice) }
            }
            LibraryEvent.OnDismissNotice -> view.update { it.copy(notice = null) }

            LibraryEvent.OnEditStart -> selectedReference()?.let { reference ->
                view.update { it.copy(editing = reference.toDraft()) }
            }
            is LibraryEvent.OnEditChange -> view.update { it.copy(editing = event.draft) }
            LibraryEvent.OnEditCancel -> view.update { it.copy(editing = null) }
            LibraryEvent.OnEditSave -> {
                val reference = selectedReference() ?: return
                val draft = view.value.editing ?: return
                act {
                    library.updateReference(draft.applyTo(reference))
                    view.update { it.copy(editing = null) }
                    Notice.Done(Res.string.notice_saved)
                }
            }
            is LibraryEvent.OnTagDraftChange -> view.update { it.copy(tagDraft = event.text) }
            LibraryEvent.OnTagAdd -> onSelected { id ->
                val names = view.value.tagDraft.split(',').map { it.trim() }.filter { it.isNotEmpty() }
                if (names.isEmpty()) return@onSelected null
                library.tag(id, names)
                view.update { it.copy(tagDraft = "") }
                null
            }
            is LibraryEvent.OnTagRemove -> onSelected { id -> library.untag(id, listOf(event.name)); null }
            is LibraryEvent.OnProjectRemove -> onSelected { id -> library.removeFromProject(id, event.path); null }
            is LibraryEvent.OnUnrelate -> onSelected { id -> library.unrelate(id, event.id); null }
            is LibraryEvent.OnNoteDraftChange -> view.update { it.copy(noteDraft = event.text) }
            LibraryEvent.OnNoteAdd -> onSelected { id ->
                val body = view.value.noteDraft.trim()
                if (body.isEmpty()) return@onSelected null
                reader.addNote(id, body)
                view.update { it.copy(noteDraft = "") }
                null
            }
            is LibraryEvent.OnNoteDelete -> act { reader.removeNote(event.id); Notice.Done(Res.string.notice_note_removed) }
            LibraryEvent.OnGetPdf -> onSelected { id ->
                val fetch = attachments.getPdf(id)
                if (fetch.downloaded) Notice.Done(Res.string.notice_pdf_downloaded, listOf(fetch.source.orEmpty()))
                else Notice.Done(Res.string.notice_pdf_present)
            }
            is LibraryEvent.OnAttachPdf -> onSelected { id ->
                val attached = attachments.attachPdf(id, event.path, strict = false)
                Notice.Done(
                    when (attached.verdict) {
                        PdfVerdict.Different, PdfVerdict.Unlikely -> Res.string.notice_pdf_other_paper
                        else -> Res.string.notice_pdf_attached
                    },
                )
            }
            LibraryEvent.OnRemovePdfs -> onSelected { id -> Notice.Done(Res.string.notice_pdfs_removed, listOf(attachments.removePdfs(id))) }
            LibraryEvent.OnOpenPublisherPage -> onSelected { id ->
                val page = attachments.browserPage(id) ?: return@onSelected Notice.Done(Res.string.notice_no_publisher_page)
                view.update { it.copy(pageToOpen = page) }
                null
            }
            LibraryEvent.OnPageOpened -> view.update { it.copy(pageToOpen = null) }
            LibraryEvent.OnRefetch -> onSelected { id ->
                val changed = library.refetch(id).count { !it.kept }
                if (changed == 0) Notice.Done(Res.string.notice_nothing_changed) else Notice.Done(Res.string.notice_refetched, listOf(changed))
            }
            LibraryEvent.OnRestore -> onSelected { id ->
                library.restoreReference(id)
                view.update { it.copy(selectedId = null, selectedIds = emptySet()) }
                Notice.Done(Res.string.notice_restored)
            }
            is LibraryEvent.OnNameDialog -> view.update { it.copy(nameDialog = event.dialog) }
            is LibraryEvent.OnNameConfirm -> {
                val dialog = view.value.nameDialog ?: return
                val name = event.name.trim()
                if (name.isEmpty()) return
                view.update { it.copy(nameDialog = null) }
                when (dialog) {
                    is NameDialog.NewProject ->
                        if (dialog.withIds.isEmpty()) onEvent(LibraryEvent.OnCreateProject(name, dialog.parentPath))
                        else onEvent(LibraryEvent.OnCreateCollectionFromSelection(name, dialog.withIds))
                    is NameDialog.RenameProject -> act {
                        val path = library.renameProject(dialog.path, name)
                        // The project in view (or one under it) moved with the rename.
                        query.update { q ->
                            val source = q.source
                            if (source is LibrarySource.InProject && source.path.isUnder(dialog.path))
                                q.copy(source = LibrarySource.InProject(path + source.path.removePrefix(dialog.path)))
                            else q
                        }
                        Notice.Done(Res.string.notice_project_renamed, listOf(name))
                    }
                    is NameDialog.SaveSearch -> act {
                        search.saveSearch(name, dialog.query)
                        reloadSavedSearches()
                        Notice.Done(Res.string.notice_search_saved, listOf(name))
                    }
                    is NameDialog.RenameSavedSearch -> act {
                        search.renameSaved(dialog.current, name)
                        reloadSavedSearches()
                        Notice.Done(Res.string.notice_project_renamed, listOf(name))
                    }
                    is NameDialog.RenameTag -> act {
                        library.renameTag(dialog.current, name)
                        query.update { q -> q.copy(tags = q.tags.map { if (it == dialog.current) name else it }.toSet()) }
                        Notice.Done(Res.string.notice_tag_renamed, listOf(name))
                    }
                }
            }
            is LibraryEvent.OnConfirm -> view.update { it.copy(confirm = event.confirm) }
            LibraryEvent.OnConfirmed -> {
                val confirm = view.value.confirm ?: return
                view.update { it.copy(confirm = null) }
                when (confirm) {
                    is Confirm.DeleteProject -> act {
                        library.deleteProject(confirm.path)
                        query.update { q ->
                            val source = q.source
                            if (source is LibrarySource.InProject && source.path.isUnder(confirm.path)) q.copy(source = LibrarySource.All) else q
                        }
                        Notice.Done(Res.string.notice_project_deleted, listOf(confirm.name))
                    }
                    is Confirm.DeleteTag -> act {
                        val count = library.deleteTag(confirm.name)
                        query.update { q -> q.copy(tags = q.tags - confirm.name) }
                        Notice.Done(Res.string.notice_tag_deleted, listOf(confirm.name, count))
                    }
                }
            }
            is LibraryEvent.OnSavedSearchClick -> savedSearches.value.firstOrNull { it.name == event.name }?.let { saved ->
                query.update { it.copy(text = saved.query, source = LibrarySource.All, tags = emptySet()) }
            }
            is LibraryEvent.OnSavedSearchRemove -> act {
                search.removeSaved(event.name)
                reloadSavedSearches()
                Notice.Done(Res.string.notice_search_removed, listOf(event.name))
            }
            is LibraryEvent.OnTagColor -> act {
                library.colorTag(event.name, event.color)
                null
            }
            LibraryEvent.OnToggleSidebar -> view.update { it.copy(isSidebarCollapsed = !it.isSidebarCollapsed) }
            is LibraryEvent.OnToggleQuickLook -> view.update { it.copy(isQuickLookOpen = event.open ?: !it.isQuickLookOpen) }
            is LibraryEvent.OnExportProject -> act {
                val count = exchange.exportTo(
                    dev.neurokitchen.quire.domain.model.ExportSelection(projectPath = event.projectId, recursive = true, format = "biblatex"),
                    event.path,
                )
                Notice.Done(Res.string.tools_exported, listOf(count, event.path))
            }
        }
    }

    /** Runs `action`; its notice, or the engine's message if it fails, goes to the state. */
    /** Runs an action and reports its notice (none if it returns `null`), or its failure. */
    private fun act(action: suspend () -> Notice?) {
        viewModelScope.launch {
            val notice = try {
                action()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Notice.Failed(e.message ?: e.toString())
            }
            if (notice != null) view.update { it.copy(notice = notice) }
        }
    }

    private fun dialog(change: (AddDialogState) -> AddDialogState) =
        view.update { options -> options.copy(addDialog = options.addDialog?.let(change)) }

    /** Runs the dialog's `action`; a failure stays in the dialog (to correct and retry). */
    /**
     * Adds each identifier of the dialog (one per line). All added: the notice; some failed: the
     * dialog keeps only those lines, with why, and `null`.
     */
    private suspend fun addIdentifiers(dialog: AddDialogState, project: String?): Notice? {
        val inputs = dialog.identifier.lines().map { it.trim() }.filter { it.isNotEmpty() }
        val outcomes = mutableListOf<AddOutcome>()
        val failed = mutableListOf<Pair<String, String>>()
        for (input in inputs) {
            try {
                outcomes += library.addByIdentifier(input, project)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                failed += input to (e.message ?: e.toString())
            }
        }
        if (failed.isNotEmpty()) {
            // A single input keeps the engine's message as it was.
            val why = failed.singleOrNull()?.second?.takeIf { inputs.size == 1 }
                ?: failed.joinToString("\n") { (input, message) -> "$input: $message" }
            dialog { it.copy(isWorking = false, identifier = failed.joinToString("\n") { it.first }, error = why) }
            return null
        }
        return outcomes.singleOrNull()?.let(::added)
            ?: Notice.Done(Res.string.notice_added_many, listOf(outcomes.count { it.isNew }, outcomes.count { !it.isNew }))
    }

    private fun working(action: suspend (AddDialogState) -> Unit) {
        val current = view.value.addDialog ?: return
        dialog { it.copy(isWorking = true, error = null) }
        viewModelScope.launch {
            try {
                action(current)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                dialog { it.copy(isWorking = false, error = e.message ?: e.toString()) }
            }
        }
    }

    private fun selection(): Set<String> = view.value.selectedIds.ifEmpty { setOfNotNull(view.value.selectedId) }

    private fun selectedId(): String? = view.value.selectedId ?: view.value.selectedIds.firstOrNull()

    private fun selectedReference(): Reference? = selectedId()?.let { id -> shown.firstOrNull { it.id == id } }

    /** [act] on the selected reference; `null` from `action` means nothing to report. */
    private fun onSelected(action: suspend (String) -> Notice?) {
        val id = selectedId() ?: return
        viewModelScope.launch {
            val notice = try {
                action(id)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Notice.Failed(e.message ?: e.toString())
            }
            if (notice != null) view.update { it.copy(notice = notice) }
        }
    }

    private fun projectInView(): String? = (query.value.source as? LibrarySource.InProject)?.path

    /** What the screen shows that is not part of the query. */
    private data class ViewOptions(
        val selectedId: String? = null,
        val selectedIds: Set<String> = emptySet(),
        val lastClickedId: String? = null,
        val expandedProjects: Set<String> = emptySet(),
        val collapsedSections: Set<ItemSection> = emptySet(),
        val isSidebarCollapsed: Boolean = false,
        val isInspectorCollapsed: Boolean = false,
        val addDialog: AddDialogState? = null,
        val nameDialog: NameDialog? = null,
        val confirm: Confirm? = null,
        val notice: Notice? = null,
        val editing: EditDraft? = null,
        val tagDraft: String = "",
        val noteDraft: String = "",
        val pageToOpen: String? = null,
        val isQuickLookOpen: Boolean = false,
    )

    private data class Extras(
        val id: String? = null,
        val notes: List<Note> = emptyList(),
        val related: List<Reference> = emptyList(),
        val pdfFiles: List<String> = emptyList(),
        val issues: List<String> = emptyList(),
    )
}

private fun Reference.toDraft() = EditDraft(
    title = title,
    authors = creators.joinToString("; ") { it.displayName },
    date = date,
    containerTitle = containerTitle,
    volume = volume,
    issue = issue,
    pages = pages,
    doi = doi,
    abstract = abstract,
)

/** The reference with the draft's fields (authors "Family, Given; …"; the year from the date). */
internal fun EditDraft.applyTo(reference: Reference): Reference {
    val creators = authors.split(';').map { it.trim() }.filter { it.isNotEmpty() }.map { name ->
        reference.creators.firstOrNull { it.displayName == name } ?: Creator(sortName = name.substringBefore(',').trim(), displayName = name)
    }
    return reference.copy(
        title = title.trim(),
        creators = creators,
        date = date.trim(),
        year = date.trim().take(4).toIntOrNull(),
        containerTitle = containerTitle.trim(),
        volume = volume.trim(),
        issue = issue.trim(),
        pages = pages.trim(),
        doi = doi.trim(),
        abstract = abstract.trim(),
    )
}

private fun added(outcome: AddOutcome) =
    Notice.Done(if (outcome.isNew) Res.string.notice_added else Res.string.notice_already_in_library, listOf(outcome.reference.title))

/** The typed-in reference: authors "Family, Given; …", the year as its date; into `project` if given. */
internal fun ManualEntry.toReference(project: String?): Reference {
    val creators = authors.split(';').map { it.trim() }.filter { it.isNotEmpty() }.map { name ->
        Creator(sortName = name.substringBefore(',').trim(), displayName = name)
    }
    val year = year.trim().toIntOrNull()
    return Reference(
        id = "",
        citeKey = "",
        kind = "article-journal",
        title = title.trim(),
        creators = creators,
        year = year,
        date = year?.toString().orEmpty(),
        containerTitle = containerTitle.trim(),
        doi = doi.trim(),
        projects = listOfNotNull(project),
    )
}

private fun <T> Set<T>.toggle(value: T): Set<T> = if (value in this) this - value else this + value

/** `thesis/chapter-2` is under `thesis` (and under itself); `thesis-old` is not. */
private fun String.isUnder(path: String) = this == path || startsWith("$path/")

/** The library's coloured tags in its order, then the other tags of the references in view, sorted. */
private fun tagRows(library: List<Tag>, references: List<Reference>, selected: Set<String>): List<TagState> {
    val colored = library.filter { it.color != null }
    val coloredNames = colored.map { it.name }.toSet()
    val others = (references.flatMap { it.tags } + selected).distinct().filter { it !in coloredNames }.sorted()
    return colored.map { TagState(it.name, it.name in selected, it.color) } + others.map { TagState(it, it in selected) }
}

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

private fun detail(
    reference: Reference,
    projects: List<Project>,
    notes: List<Note>,
    related: List<Reference>,
    pdfFiles: List<String>,
    issues: List<String>,
    isTrashed: Boolean,
): ReferenceDetailState {
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
        projectPaths = reference.projects,
        notes = notes.map { NoteState(it.id, it.body, it.updated) },
        related = related.map { r ->
            val card = ReferenceCardState.from(r)
            RelatedState(r.id, r.title, listOf(card.creator, card.year).filter { it.isNotBlank() }.joinToString(" · "))
        },
        pdfFiles = pdfFiles,
        issues = issues,
        isTrashed = isTrashed,
    )
}

