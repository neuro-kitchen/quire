package dev.neurokitchen.quire.presentation.screens.reader

import kotlin.test.assertTrue
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.domain.model.PdfMatch
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.PageRect
import dev.neurokitchen.quire.domain.model.Highlight
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

    @Test
    fun findInTheTextCountsAndGoesAround() = runTest(dispatcher) {
        val vm = viewModel()
        val text = vm.state.value.tabs[1].blocks.joinToString("\n") { it.text }
        val expected = Regex("place", RegexOption.IGNORE_CASE).findAll(text).count()
        vm.onEvent(ReaderEvent.OnFindToggle)
        vm.onEvent(ReaderEvent.OnFindChange("PLACE"))
        advanceUntilIdle()
        val find = vm.state.value.find
        assertEquals(expected, find.count(ReaderMode.Text))
        assertTrue(expected > 1, "the sample text says place more than once")
        vm.onEvent(ReaderEvent.OnFindPrevious)
        assertEquals(expected - 1, vm.state.value.find.current, "back from the first is the last")
        vm.onEvent(ReaderEvent.OnFindNext)
        assertEquals(0, vm.state.value.find.current)
        vm.onEvent(ReaderEvent.OnTabClick(0))
        val abstract = vm.state.value.tabs[0].blocks.joinToString("\n") { it.text }
        assertEquals(Regex("place", RegexOption.IGNORE_CASE).findAll(abstract).count(), vm.state.value.find.count(ReaderMode.Text), "counted again in the tab shown")
    }

    @Test
    fun findOnThePdfAsksTheEngineAndTheTextLayerPassesThrough() = runTest(dispatcher) {
        val withPdf = object : ReaderRepository by reader {
            override suspend fun pdfFiles(id: String) = listOf("/papers/okeefe.pdf")
        }
        val searched = mutableListOf<String>()
        val pdf = object : PdfRepository by SamplePdfRepository() {
            override suspend fun pageSizes(path: String) = listOf(PageSize(612f, 792f), PageSize(612f, 792f))
            override suspend fun locate(path: String, highlights: List<Highlight>) = emptyList<QuoteLocation>()
            override suspend fun search(path: String, query: String, limit: Int): List<PdfMatch> {
                searched += query
                return listOf(PdfMatch(0, listOf(PageRect(72f, 60f, 120f, 72f))), PdfMatch(1, listOf(PageRect(72f, 90f, 120f, 102f))))
            }
            override suspend fun nearestGlyph(path: String, page: Int, x: Float, y: Float) = 7
        }
        val vm = ReaderViewModel("3", "The hippocampus as a spatial map", withPdf, pdf).also { advanceUntilIdle() }
        assertEquals(ReaderMode.Pdf, vm.state.value.mode)
        vm.onEvent(ReaderEvent.OnFindChange("pla"))
        vm.onEvent(ReaderEvent.OnFindChange("place"))
        advanceUntilIdle()
        assertEquals(listOf("place"), searched, "typing quickly searches once, for the last text")
        assertEquals(2, vm.state.value.find.count(ReaderMode.Pdf))
        vm.onEvent(ReaderEvent.OnFindNext)
        assertEquals(1, vm.state.value.find.current)
        assertEquals(7, vm.glyphAt(0, 80f, 66f))
    }
}
