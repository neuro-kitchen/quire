package dev.neurokitchen.quire.presentation.screens.library

import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.data.repository.SampleReaderRepository
import dev.neurokitchen.quire.domain.repository.AttachmentRepository
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.domain.repository.SearchRepository
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.SavedSearch
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import dev.neurokitchen.quire.domain.model.AttachedPdf
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.PdfVerdict
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.SortColumn
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.notice_added
import dev.neurokitchen.quire.resources.notice_added_to_project
import dev.neurokitchen.quire.resources.notice_imported
import dev.neurokitchen.quire.resources.notice_pdf_other_paper
import dev.neurokitchen.quire.resources.notice_project_renamed
import dev.neurokitchen.quire.resources.notice_saved
import dev.neurokitchen.quire.resources.notice_search_saved
import dev.neurokitchen.quire.resources.notice_trashed
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class LibraryViewModelTest {
    @BeforeTest
    fun setUp() = Dispatchers.setMain(UnconfinedTestDispatcher())

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    /** Pasted BibTeX it was given, with the project it was for. */
    private class RecordingExchange : ExchangeRepository {
        val imported = mutableListOf<Pair<String, String?>>()
        override suspend fun importBibText(text: String, projectPath: String?): Int {
            imported += text to projectPath
            return 2
        }
    }

    private val exchange = RecordingExchange()

    /** Attaches whatever it is given as another paper's PDF, and knows one publisher page. */
    private class FakeAttachments : AttachmentRepository {
        val attached = mutableListOf<Pair<String, String>>()
        override suspend fun attachPdf(id: String, path: String, strict: Boolean): AttachedPdf {
            attached += id to path
            return AttachedPdf(Reference(id = id, citeKey = id, kind = "article-journal", title = "", creators = emptyList(), year = null), isNew = true, path = path, verdict = PdfVerdict.Different)
        }
        override suspend fun browserPage(id: String) = "https://doi.org/10.1038/nature03721"
    }

    private val attachments = FakeAttachments()

    /** Saved searches in memory. */
    private class FakeSearch : SearchRepository {
        val saved = mutableListOf<SavedSearch>()
        override suspend fun saveSearch(name: String, query: String) = SavedSearch(name, query).also { saved += it }
        override suspend fun savedSearches() = saved.toList()
        override suspend fun removeSaved(name: String) {
            saved.removeAll { it.name == name }
        }
    }

    private val search = FakeSearch()

    private fun viewModel(library: LibraryRepository = SampleLibraryRepository()) =
        LibraryViewModel(library, SampleReaderRepository(), exchange, attachments, search)

    private suspend fun LibraryViewModel.settled(check: (LibraryState) -> Boolean = { true }) =
        state.first { !it.isLoading && check(it) }

    @Test
    fun searchNarrowsTheList() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnQueryChange("hubel"))
        assertEquals(listOf("hubel1962"), vm.settled { it.query == "hubel" }.references.map { it.citeKey })
    }

    @Test
    fun aProjectShowsOnlyItsReferencesAndIsMarkedSelected() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject("thesis/chapter-2")))
        vm.onEvent(LibraryEvent.OnProjectExpandToggle("thesis"))
        val state = vm.settled { it.source == LibrarySource.InProject("thesis/chapter-2") && it.projects.size == 4 }
        assertEquals(setOf("okeefe1971", "hafting2005"), state.references.map { it.citeKey }.toSet())
        assertEquals(listOf("thesis/chapter-2"), state.projects.filter { it.isSelected }.map { it.id })
        assertEquals(1, state.projects.first { it.id == "thesis/chapter-2" }.depth)
    }

    @Test
    fun childProjectsAreHiddenUntilTheParentIsExpanded() = runTest {
        val vm = viewModel()
        assertTrue(vm.settled().projects.none { it.id == "thesis/chapter-2" })
    }

    @Test
    fun tagsNarrowAndSortingFlips() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnTagClick("deep learning"))
        vm.onEvent(LibraryEvent.OnSortClick(SortColumn.Year))
        assertEquals(listOf("lecun2015", "he2016", "vaswani2017"), vm.settled { it.sort == SortColumn.Year && it.ascending }.references.map { it.citeKey })
        vm.onEvent(LibraryEvent.OnSortClick(SortColumn.Year))
        assertEquals(listOf("vaswani2017", "he2016", "lecun2015"), vm.settled { !it.ascending }.references.map { it.citeKey })
    }

    @Test
    fun selectingAReferenceFillsTheItemPaneUntilClosed() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnReferenceClick("3"))
        val selected = vm.settled { it.selected != null }.selected!!
        assertEquals("okeefe1971", selected.citeKey)
        assertEquals(listOf("Chapter 2 – Navigation"), selected.projects)
        vm.onEvent(LibraryEvent.OnCloseDetail)
        assertNull(vm.settled { it.selected == null }.selected)
    }

    @Test
    fun sectionsCollapseAndExpand() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnSectionToggle(ItemSection.Abstract))
        assertEquals(setOf(ItemSection.Abstract), vm.settled { it.collapsedSections.isNotEmpty() }.collapsedSections)
        vm.onEvent(LibraryEvent.OnSectionToggle(ItemSection.Abstract))
        assertTrue(vm.settled { it.collapsedSections.isEmpty() }.collapsedSections.isEmpty())
    }

    @Test
    fun aTypedReferenceIsAddedWithItsTagsAndReported() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnShowAddDialog(true))
        vm.onEvent(LibraryEvent.OnAddTabSelect(AddTab.Manual))
        vm.onEvent(LibraryEvent.OnManualChange(ManualEntry(title = "Grid cells", authors = "Hafting, Torkel", year = "2005")))
        assertTrue(vm.settled().addDialog!!.canAdd)
        vm.onEvent(LibraryEvent.OnAddConfirm)
        val state = vm.settled { it.addDialog == null && it.notice != null }
        assertEquals(Notice.Done(Res.string.notice_added, listOf("Grid cells")), state.notice)
        assertTrue(state.references.any { it.title == "Grid cells" && it.creator == "Hafting" })
        vm.onEvent(LibraryEvent.OnDismissNotice)
        assertNull(vm.settled().notice)
    }

    @Test
    fun pastedBibtexGoesToTheProjectInView() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject("thesis")))
        vm.onEvent(LibraryEvent.OnShowAddDialog(true))
        vm.onEvent(LibraryEvent.OnAddTabSelect(AddTab.Bibtex))
        vm.onEvent(LibraryEvent.OnBibtexChange("@article{a, title={A}}"))
        vm.onEvent(LibraryEvent.OnAddConfirm)
        assertEquals(Notice.Done(Res.string.notice_imported, listOf(2)), vm.settled { it.notice != null }.notice)
        assertEquals(listOf<Pair<String, String?>>("@article{a, title={A}}" to "thesis"), exchange.imported)
    }

    /** The sample repository has no engine: a lookup fails, and the message stays in the dialog. */
    @Test
    fun aFailedLookupStaysInTheDialog() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnShowAddDialog(true))
        vm.onEvent(LibraryEvent.OnIdentifierChange("10.1038/nature03721"))
        vm.onEvent(LibraryEvent.OnLookup)
        val dialog = vm.settled { it.addDialog?.error != null }.addDialog!!
        assertEquals("needs the engine", dialog.error)
        assertTrue(!dialog.isWorking && dialog.preview == null)
        assertNull(vm.settled().notice, "not a notice: the dialog is still open")
    }

    @Test
    fun aSelectionGoesIntoANewProjectAndIntoAnExistingOne() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnCreateCollectionFromSelection("Grid", setOf("3")))
        vm.settled { it.notice != null }
        vm.onEvent(LibraryEvent.OnBatchAddToProject(setOf("2"), "grid"))
        vm.settled { it.notice == Notice.Done(Res.string.notice_added_to_project, listOf(1, "grid")) }
        vm.onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject("grid")))
        val inGrid = vm.settled { it.source == LibrarySource.InProject("grid") && it.references.size == 2 }
        assertEquals(setOf("hubel1962", "okeefe1971"), inGrid.references.map { it.citeKey }.toSet())
    }

    @Test
    fun deletingTheSelectionReportsHowMany() = runTest {
        val vm = viewModel()
        val ids = vm.settled().references.take(2).map { it.id }
        ids.forEach { vm.onEvent(LibraryEvent.OnReferenceSelect(it, isCtrlPressed = true)) }
        vm.onEvent(LibraryEvent.OnDeleteSelected)
        val state = vm.settled { it.notice != null }
        assertEquals(Notice.Done(Res.string.notice_trashed, listOf(2)), state.notice)
        assertTrue(state.references.none { it.id in ids })
    }

    @Test
    fun tagsAreAddedCommaSeparatedAndRemoved() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnReferenceClick("3"))
        vm.settled { it.selected != null }
        vm.onEvent(LibraryEvent.OnTagDraftChange("grid, place cells "))
        vm.onEvent(LibraryEvent.OnTagAdd)
        val tagged = vm.settled { "grid" in it.selected?.tags.orEmpty() }
        assertTrue("place cells" in tagged.selected!!.tags)
        assertEquals("", tagged.tagDraft)
        vm.onEvent(LibraryEvent.OnTagRemove("grid"))
        assertTrue("grid" !in vm.settled { "grid" !in it.selected?.tags.orEmpty() }.selected!!.tags)
    }

    @Test
    fun anEditIsSavedFromTheDraft() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnReferenceClick("3"))
        vm.settled { it.selected != null }
        vm.onEvent(LibraryEvent.OnEditStart)
        val draft = vm.settled { it.editing != null }.editing!!
        vm.onEvent(LibraryEvent.OnEditChange(draft.copy(title = "The hippocampus as a spatial map")))
        vm.onEvent(LibraryEvent.OnEditSave)
        val state = vm.settled { it.notice != null }
        assertEquals(Notice.Done(Res.string.notice_saved), state.notice)
        assertNull(state.editing)
        assertEquals("The hippocampus as a spatial map", state.selected!!.title)
    }

    @Test
    fun aPdfOfAnotherPaperIsAttachedWithAWarning() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnReferenceClick("3"))
        vm.settled { it.selected != null }
        vm.onEvent(LibraryEvent.OnAttachPdf("/papers/x.pdf"))
        assertEquals(Res.string.notice_pdf_other_paper, (vm.settled { it.notice != null }.notice as Notice.Done).message)
        assertEquals(listOf("3" to "/papers/x.pdf"), attachments.attached)
    }

    @Test
    fun thePublisherPageIsOpenedOnce() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnReferenceClick("3"))
        vm.settled { it.selected != null }
        vm.onEvent(LibraryEvent.OnOpenPublisherPage)
        assertEquals("https://doi.org/10.1038/nature03721", vm.settled { it.pageToOpen != null }.pageToOpen)
        vm.onEvent(LibraryEvent.OnPageOpened)
        assertNull(vm.settled { it.pageToOpen == null }.pageToOpen)
    }

    @Test
    fun theTablesNewCollectionHoldsTheSelection() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnNameDialog(NameDialog.NewProject(withIds = setOf("2", "3"))))
        vm.onEvent(LibraryEvent.OnNameConfirm("Grid"))
        vm.settled { it.notice != null }
        assertNull(vm.settled().nameDialog)
        vm.onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject("grid")))
        val inGrid = vm.settled { it.source == LibrarySource.InProject("grid") && it.references.size == 2 }
        assertEquals(setOf("hubel1962", "okeefe1971"), inGrid.references.map { it.citeKey }.toSet())
    }

    @Test
    fun renamingAProjectKeepsItsSubprojectInView() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject("thesis/chapter-2")))
        vm.onEvent(LibraryEvent.OnNameDialog(NameDialog.RenameProject("thesis", "Thesis")))
        vm.onEvent(LibraryEvent.OnNameConfirm("PhD"))
        val state = vm.settled { it.source == LibrarySource.InProject("phd/chapter-2") }
        assertEquals(setOf("okeefe1971", "hafting2005"), state.references.map { it.citeKey }.toSet())
        assertEquals(Notice.Done(Res.string.notice_project_renamed, listOf("PhD")), state.notice)
    }

    @Test
    fun aProjectIsDeletedOnlyOnceConfirmed() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject("thesis")))
        vm.onEvent(LibraryEvent.OnConfirm(Confirm.DeleteProject("thesis", "Thesis")))
        assertTrue(vm.settled().projects.any { it.id == "thesis" }, "nothing happens before confirming")
        vm.onEvent(LibraryEvent.OnConfirmed)
        val state = vm.settled { it.notice != null }
        assertEquals(LibrarySource.All, state.source)
        assertTrue(state.projects.none { it.id == "thesis" })
        assertNull(state.confirm)
    }

    @Test
    fun colouredTagsComeFirstAndCanBeRecolouredRenamedAndDeleted() = runTest {
        val vm = viewModel()
        val first = vm.settled { it.tags.isNotEmpty() }.tags.first()
        assertEquals(TagState("deep learning", isSelected = false, color = "#2EA8E5"), first)
        vm.onEvent(LibraryEvent.OnTagColor("vision", "#FF6666"))
        assertEquals(listOf("deep learning", "vision"), vm.settled { s -> s.tags.count { it.color != null } == 2 }.tags.take(2).map { it.name })
        vm.onEvent(LibraryEvent.OnTagClick("vision"))
        vm.onEvent(LibraryEvent.OnNameDialog(NameDialog.RenameTag("vision")))
        vm.onEvent(LibraryEvent.OnNameConfirm("visual cortex"))
        val renamed = vm.settled { s -> s.tags.any { it.name == "visual cortex" } }
        assertTrue(renamed.tags.first { it.name == "visual cortex" }.let { it.isSelected && it.color == "#FF6666" })
        vm.onEvent(LibraryEvent.OnConfirm(Confirm.DeleteTag("visual cortex")))
        vm.onEvent(LibraryEvent.OnConfirmed)
        val deleted = vm.settled { s -> s.tags.none { it.name == "visual cortex" } }
        assertTrue(deleted.notice is Notice.Done)
    }

    /** The engine refuses `status:r` (a filter half typed): the screen keeps going, with the last results. */
    @Test
    fun aSearchTheEngineRefusesKeepsTheLastResults() = runTest {
        val sample = SampleLibraryRepository()
        val strict = object : LibraryRepository by sample {
            override fun references(query: LibraryQuery): Flow<List<Reference>> =
                if (query.text == "status:r") flow { throw IllegalArgumentException("status:r: unread, reading or read") }
                else sample.references(query)
        }
        val vm = viewModel(strict)
        vm.onEvent(LibraryEvent.OnQueryChange("hubel"))
        vm.settled { it.query == "hubel" && it.references.size == 1 }
        vm.onEvent(LibraryEvent.OnQueryChange("status:r"))
        val refused = vm.settled { it.searchError != null }
        assertEquals("status:r: unread, reading or read", refused.searchError)
        assertEquals(listOf("hubel1962"), refused.references.map { it.citeKey })
        assertNull(refused.error, "not the library's error screen")
        vm.onEvent(LibraryEvent.OnQueryChange(""))
        val recovered = vm.settled { it.query == "" && it.searchError == null }
        assertTrue(recovered.references.size > 1)
    }

    @Test
    fun aSavedSearchRunsOverTheWholeLibraryAndShowsAsActive() = runTest {
        val vm = viewModel()
        vm.onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject("thesis")))
        vm.onEvent(LibraryEvent.OnQueryChange("hubel"))
        vm.onEvent(LibraryEvent.OnNameDialog(NameDialog.SaveSearch("hubel")))
        vm.onEvent(LibraryEvent.OnNameConfirm("Hubel"))
        assertEquals(Notice.Done(Res.string.notice_search_saved, listOf("Hubel")), vm.settled { it.savedSearches.isNotEmpty() }.notice)
        vm.onEvent(LibraryEvent.OnQueryChange(""))
        assertTrue(vm.settled { it.query == "" }.savedSearches.none { it.isActive })
        vm.onEvent(LibraryEvent.OnSavedSearchClick("Hubel"))
        val running = vm.settled { it.query == "hubel" }
        assertEquals(LibrarySource.All, running.source)
        assertEquals(listOf(SavedSearchState("Hubel", "hubel", isActive = true)), running.savedSearches)
        vm.onEvent(LibraryEvent.OnSavedSearchRemove("Hubel"))
        assertTrue(vm.settled { it.savedSearches.isEmpty() }.savedSearches.isEmpty())
    }
}
