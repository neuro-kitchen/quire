package dev.neurokitchen.quire.presentation.screens.reader

import kotlinx.coroutines.test.runCurrent
import dev.neurokitchen.quire.presentation.screens.reader.components.ExtractedFigure
import dev.neurokitchen.quire.domain.repository.ReaderLayout
import dev.neurokitchen.quire.domain.repository.GuiPreferences
import dev.neurokitchen.quire.data.repository.SamplePreferencesRepository
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
import kotlinx.coroutines.delay
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.advanceUntilIdle
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.domain.model.PdfFigure
import kotlin.test.assertNotNull
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

    private val withPdf = object : ReaderRepository by reader {
        override suspend fun pdfFiles(id: String) = listOf("/papers/okeefe.pdf")
    }

    /** A PDF of `pages` pages whose renders are counted, as (page, width). */
    private class CountingPdf(private val pages: Int = 5, private val sizesDelay: Long = 0) : PdfRepository by SamplePdfRepository() {
        val renders = mutableListOf<Pair<Int, Int>>()
        override suspend fun pageSizes(path: String): List<PageSize> {
            delay(sizesDelay)
            return List(pages) { PageSize(612f, 792f) }
        }
        override suspend fun locate(path: String, highlights: List<Highlight>) = emptyList<QuoteLocation>()
        override suspend fun render(path: String, index: Int, width: Int): PageImage {
            renders += index to width
            val height = width * 792 / 612
            return PageImage(width, height, ByteArray(width * height * 4))
        }
    }

    /** The text once waited for the PDF's page sizes (and the highlights' places) before loading. */
    @Test
    fun theTextDoesNotWaitForThePdf() = runTest(dispatcher) {
        val vm = ReaderViewModel("3", "The hippocampus as a spatial map", withPdf, CountingPdf(sizesDelay = 5_000), dispatcher)
        advanceTimeBy(100)
        assertTrue(vm.state.value.tabs.isNotEmpty(), "the text is there while the PDF is still opening")
        assertTrue(vm.state.value.pdf.isLoading)
        advanceUntilIdle()
        assertEquals(5, vm.state.value.pdf.pages.size)
        assertEquals(false, vm.state.value.pdf.isLoading)
    }

    /** A page was rendered again for every pixel of width (each resize, each zoom step). */
    @Test
    fun nearbyWidthsShareOneRenderAndLaterPagesRenderAhead() = runTest(dispatcher) {
        val pdf = CountingPdf()
        val vm = ReaderViewModel("3", "The hippocampus as a spatial map", withPdf, pdf, dispatcher).also { advanceUntilIdle() }
        assertNotNull(vm.page(0, 1000))
        assertNotNull(vm.page(0, 1013))
        advanceUntilIdle()
        assertEquals(1, pdf.renders.count { it.first == 0 }, "one render for both widths: ${pdf.renders}")
        assertEquals(setOf(0, 1, 2), pdf.renders.map { it.first }.toSet(), "the next two pages rendered ahead")
        assertNotNull(vm.page(1, 1000))
        advanceUntilIdle()
        assertEquals(1, pdf.renders.count { it.first == 1 }, "page 2 came from the render ahead")
    }

    /**
     * Papers read from their PDF have no figure images; the drawer shows figures cropped from the
     * pages. They are looked for only once the drawer opens (it renders pages).
     */
    @Test
    fun pdfFiguresLoadWhenTheDrawerOpensAndAreCroppedFromThePage() = runTest(dispatcher) {
        var asked = 0
        val figure = PdfFigure(1, "Figure 2", "Figure 2. Firing.", PageRect(72f, 100f, 378f, 253f))
        val pdf = object : PdfRepository by CountingPdf() {
            override suspend fun figures(path: String): List<PdfFigure> = listOf(figure).also { asked++ }
        }
        val vm = ReaderViewModel("3", "The hippocampus as a spatial map", withPdf, pdf, dispatcher).also { advanceUntilIdle() }
        assertEquals(0, asked, "not before the drawer opens")
        vm.onEvent(ReaderEvent.OnToggleFigureDrawer)
        advanceUntilIdle()
        assertEquals(listOf(figure), vm.state.value.pdf.figures)
        val crop = assertNotNull(vm.pdfFigure(figure))
        // The page at 1536 px for its 612 pt: 306 x 153 pt of figure.
        assertEquals(768, crop.width)
        assertEquals(384, crop.height)
        vm.onEvent(ReaderEvent.OnToggleFigureDrawer)
        vm.onEvent(ReaderEvent.OnToggleFigureDrawer)
        advanceUntilIdle()
        assertEquals(1, asked, "looked for once")
    }

    /** While a page renders at a new width (zooming in), the nearest width already rendered shows. */
    @Test
    fun aRenderedWidthStandsInForAnother() = runTest(dispatcher) {
        val vm = ReaderViewModel("3", "The hippocampus as a spatial map", withPdf, CountingPdf(), dispatcher).also { advanceUntilIdle() }
        kotlin.test.assertNull(vm.renderedNear(0, 2000))
        vm.page(0, 1000)
        assertNotNull(vm.renderedNear(0, 2000))
    }

    /** Pane sizes were kept only while shown and lost with the window; they live in the preferences now. */
    @Test
    fun paneSizesComeFromThePreferencesAndAreSavedThere() = runTest(dispatcher) {
        val saved = ReaderLayout(annotationsWidth = 300, notesWidth = 240, two = listOf(0.3f, 0.7f))
        val preferences = SamplePreferencesRepository(GuiPreferences(readerLayout = saved))
        val vm = ReaderViewModel("3", "A paper", reader, SamplePdfRepository(), preferences = preferences).also { advanceUntilIdle() }
        assertEquals(saved, vm.state.value.layout)
        val dragged = saved.copy(two = listOf(0.6f, 0.4f))
        vm.onEvent(ReaderEvent.OnLayoutChange(saved.copy(two = listOf(0.4f, 0.6f))))
        vm.onEvent(ReaderEvent.OnLayoutChange(dragged))
        assertEquals(dragged, vm.state.value.layout, "the screen follows the drag at once")
        runCurrent()
        assertEquals(saved, preferences.preferences.value.readerLayout, "written once the drag stops, not on every move")
        advanceUntilIdle()
        assertEquals(dragged, preferences.preferences.value.readerLayout)
    }

    /** A figure or a table could only sit in the drawer; it can be read beside the text or the PDF. */
    @Test
    fun aFigureIsPinnedBesideTheTextAndUnpinned() = runTest(dispatcher) {
        val vm = viewModel()
        val figure = ExtractedFigure(id = "f1", title = "Figure 1", caption = "Place fields", imageUrl = "fig1.png")
        vm.onEvent(ReaderEvent.OnPin(Pinned.Figure(figure)))
        assertEquals(Pinned.Figure(figure), vm.state.value.pinned)
        assertEquals(listOf(ReaderPane.Text, ReaderPane.Pinned), vm.state.value.panes)
        vm.onEvent(ReaderEvent.OnPin(null))
        assertEquals(listOf(ReaderPane.Text), vm.state.value.panes)
    }
}

