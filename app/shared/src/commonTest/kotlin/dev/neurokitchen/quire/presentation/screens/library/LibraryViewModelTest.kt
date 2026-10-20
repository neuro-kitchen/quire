package dev.neurokitchen.quire.presentation.screens.library

import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.domain.model.LibrarySource
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
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class LibraryViewModelTest {
    @BeforeTest
    fun setUp() = Dispatchers.setMain(UnconfinedTestDispatcher())

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    private fun viewModel() = LibraryViewModel(SampleLibraryRepository())

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
}
