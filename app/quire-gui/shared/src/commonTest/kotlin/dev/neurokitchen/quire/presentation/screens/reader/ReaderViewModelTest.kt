package dev.neurokitchen.quire.presentation.screens.reader

import dev.neurokitchen.quire.data.repository.SamplePdfRepository
import dev.neurokitchen.quire.data.repository.SampleReaderRepository
import dev.neurokitchen.quire.domain.model.PaperTabKind
import dev.neurokitchen.quire.domain.model.ReadingStatus
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

@OptIn(ExperimentalCoroutinesApi::class)
class ReaderViewModelTest {
    private val dispatcher = StandardTestDispatcher()

    @BeforeTest
    fun setUp() = Dispatchers.setMain(dispatcher)

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    private val reader = SampleReaderRepository()

    private fun TestScope.viewModel() =
        ReaderViewModel("3", "The hippocampus as a spatial map", reader, SamplePdfRepository()).also { advanceUntilIdle() }

    @Test
    fun opensOnThePapersTextWithoutAPdf() = runTest(dispatcher) {
        val state = viewModel().state.value
        assertEquals(listOf(PaperTabKind.Abstract, PaperTabKind.Paper), state.tabs.map { it.kind })
        assertEquals(1, state.selectedTab, "the paper's text, not its abstract")
        assertEquals(ReaderMode.Text, state.mode)
        assertNull(state.pdf.path)
        assertEquals(ReadingStatus.Reading, state.reading, "opening marks it as being read")
    }

    @Test
    fun selectedTextBecomesAHighlightWithAComment() = runTest(dispatcher) {
        val vm = viewModel()
        vm.onEvent(ReaderEvent.OnHighlight("  form a map of space  ", "Together they "))
        advanceUntilIdle()
        val highlight = vm.state.value.highlights.single()
        assertEquals("form a map of space", highlight.quote, "trimmed")
        vm.onEvent(ReaderEvent.OnHighlightComment(highlight.id, "the key claim"))
        advanceUntilIdle()
        assertEquals("the key claim", vm.state.value.highlights.single().comment)
        vm.onEvent(ReaderEvent.OnHighlightRemove(highlight.id))
        advanceUntilIdle()
        assertEquals(emptyList(), vm.state.value.highlights)
    }

    @Test
    fun notesAreAddedFromTheDraft() = runTest(dispatcher) {
        val vm = viewModel()
        vm.onEvent(ReaderEvent.OnNoteDraftChange("Compare with grid cells"))
        vm.onEvent(ReaderEvent.OnNoteAdd)
        advanceUntilIdle()
        assertEquals(listOf("Compare with grid cells"), vm.state.value.notes.map { it.body })
        assertEquals("", vm.state.value.noteDraft)
    }

    @Test
    fun zoomStaysInBounds() = runTest(dispatcher) {
        val vm = viewModel()
        repeat(20) { vm.onEvent(ReaderEvent.OnZoomIn) }
        assertEquals(4f, vm.state.value.pdf.zoom)
        vm.onEvent(ReaderEvent.OnZoomReset)
        assertEquals(1f, vm.state.value.pdf.zoom)
    }
}
