package dev.neurokitchen.quire.presentation.desktop

import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.advanceUntilIdle
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
class DesktopViewModelTest {
    private val dispatcher = StandardTestDispatcher()

    @BeforeTest
    fun setUp() = Dispatchers.setMain(dispatcher)

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun papersOpenOnceAndClosingMovesLeft() {
        val vm = DesktopViewModel()
        vm.onEvent(DesktopEvent.OnOpenPaper("a", "A"))
        vm.onEvent(DesktopEvent.OnOpenPaper("b", "B"))
        vm.onEvent(DesktopEvent.OnOpenPaper("a", "A"))
        assertEquals(listOf("a", "b"), vm.state.value.papers.map { it.id }, "no second tab for a")
        assertEquals("a", vm.state.value.selected)

        vm.onEvent(DesktopEvent.OnSelectTab("b"))
        vm.onEvent(DesktopEvent.OnCloseTab("b"))
        assertEquals("a", vm.state.value.selected, "its left neighbour")
        vm.onEvent(DesktopEvent.OnCloseTab("a"))
        assertNull(vm.state.value.selected, "back to the library")
    }

    /**
     * Every reference was loaded at startup, and again after every change, for the command palette
     * and the citation graph, next to the library screen's own list. Now only while one is open.
     */
    @Test
    fun allReferencesLoadOnlyWhileThePaletteOrTheGraphIsOpen() = runTest(dispatcher) {
        val base = SampleLibraryRepository()
        var streams = 0
        val library = object : LibraryRepository by base {
            override fun references(query: LibraryQuery) = base.references(query).onStart { streams++ }
        }
        val vm = DesktopViewModel(libraryRepository = library)
        advanceUntilIdle()
        assertEquals(0, streams, "nothing loaded at startup")
        vm.onEvent(DesktopEvent.OnToggleCommandPalette(true))
        advanceUntilIdle()
        assertEquals(1, streams)
        assertTrue(vm.state.value.allReferences.isNotEmpty())
        vm.onEvent(DesktopEvent.OnToggleCommandPalette(false))
        advanceUntilIdle()
        vm.onEvent(DesktopEvent.OnToggleCitationGraph(true))
        advanceUntilIdle()
        assertEquals(2, streams, "the graph loads them again while it is open")
    }

    /** Stale paper texts are rebuilt in the background a moment after startup, once. */
    @Test
    fun staleTextsAreRebuiltAfterStartup() = runTest(dispatcher) {
        var rebuilds = 0
        val library = object : LibraryRepository by SampleLibraryRepository() {
            override suspend fun rebuildStaleTexts(): Int = 1.also { rebuilds++ }
        }
        DesktopViewModel(libraryRepository = library)
        advanceTimeBy(1_000)
        assertEquals(0, rebuilds, "not while the first screen loads")
        advanceUntilIdle()
        assertEquals(1, rebuilds)
    }
}
