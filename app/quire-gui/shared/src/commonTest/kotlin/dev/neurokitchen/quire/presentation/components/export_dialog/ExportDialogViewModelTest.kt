package dev.neurokitchen.quire.presentation.components.export_dialog

import dev.neurokitchen.quire.domain.model.ExportOptions
import dev.neurokitchen.quire.domain.model.ExportProfile
import dev.neurokitchen.quire.domain.model.ExportSelection
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tools_exported
import dev.neurokitchen.quire.resources.tools_kept_updated
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
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ExportDialogViewModelTest {
    @BeforeTest
    fun setUp() = Dispatchers.setMain(UnconfinedTestDispatcher())

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    /** Exports what it is asked, as text naming the selection; records files written. */
    private class FakeExchange : ExchangeRepository {
        val asked = mutableListOf<ExportSelection>()
        val written = mutableListOf<Pair<String, ExportSelection>>()
        val kept = mutableListOf<Pair<String, ExportSelection>>()
        override fun exportFormats() = listOf("biblatex", "bibtex", "csl-json", "hayagriva")
        override fun exportEncodings() = listOf("utf-8", "utf-8-bom", "windows-1252")
        override suspend fun exportProfiles() =
            listOf(ExportProfile("typst", ExportOptions(format = "biblatex", omit = listOf("abstract", "file"), preprintsAsArticles = true)))
        override suspend fun export(selection: ExportSelection): Pair<String, Int> {
            asked += selection
            if (selection.options?.encoding == "windows-1252" && selection.options.format == "csl-json") throw IllegalStateException("Windows-1252 is for .bib files")
            return "${selection.options?.format ?: selection.profile} ${selection.options?.omit}" to 3
        }
        override suspend fun exportTo(selection: ExportSelection, path: String): Int {
            written += path to selection
            return 3
        }
        override suspend fun keepUpdated(selection: ExportSelection, path: String): Int {
            kept += path to selection
            return 3
        }
    }

    private val exchange = FakeExchange()

    private suspend fun ExportDialogViewModel.settled(check: (ExportDialogState) -> Boolean = { true }) =
        state.first { !it.isWorking && check(it) }

    /** The old export was a menu item writing BibLaTeX straight away: no choice, no look first. */
    @Test
    fun thePreviewFollowsTheOptions() = runTest {
        val vm = ExportDialogViewModel(ExportTarget(projectPath = "thesis"), exchange)
        assertEquals("biblatex []", vm.settled { it.preview != null }.preview)
        vm.onEvent(ExportDialogEvent.OnFormat("bibtex"))
        vm.onEvent(ExportDialogEvent.OnOmitToggle(OmitField.Abstract))
        vm.onEvent(ExportDialogEvent.OnMonthMacros(true))
        val state = vm.settled { it.preview == "bibtex [abstract]" }
        assertEquals(3, state.count)
        val last = exchange.asked.last()
        assertEquals("thesis", last.projectPath)
        assertEquals(ExportOptions(format = "bibtex", omit = listOf("abstract"), monthMacros = true), last.options)
    }

    /** Fields are left out by the format's own names (CSL-JSON's `keyword`, `URL`). */
    @Test
    fun omittedFieldsUseTheFormatsNames() = runTest {
        val vm = ExportDialogViewModel(ExportTarget(), exchange)
        vm.onEvent(ExportDialogEvent.OnOmitToggle(OmitField.Keywords))
        vm.onEvent(ExportDialogEvent.OnOmitToggle(OmitField.Url))
        vm.onEvent(ExportDialogEvent.OnFormat("csl-json"))
        vm.settled { it.preview?.startsWith("csl-json") == true }
        assertEquals(listOf("keyword", "URL"), exchange.asked.last().options!!.omit)
    }

    /** A profile fills the options; it is sent by name until an option is changed. */
    @Test
    fun aProfileFillsTheOptionsUntilChanged() = runTest {
        val vm = ExportDialogViewModel(ExportTarget(), exchange)
        vm.onEvent(ExportDialogEvent.OnProfile("typst"))
        val state = vm.settled { it.profile == "typst" && it.preview != null }
        assertEquals(setOf(OmitField.Abstract, OmitField.File), state.omitted)
        assertTrue(state.options.preprintsAsArticles)
        assertEquals("typst" to null, exchange.asked.last().let { it.profile to it.options })
        vm.onEvent(ExportDialogEvent.OnOmitToggle(OmitField.File))
        val changed = vm.settled { it.profile == null }
        assertEquals(setOf(OmitField.Abstract), changed.omitted)
        assertEquals(null to listOf("abstract"), exchange.asked.last().let { it.profile to it.options?.omit })
    }

    /** ASCII is an encoding choice for .bib (LaTeX commands); other formats offer UTF-8 only. */
    @Test
    fun encodingsFitTheFormat() = runTest {
        val vm = ExportDialogViewModel(ExportTarget(), exchange)
        assertEquals(listOf(ENCODING_UTF8, ENCODING_UTF8_BOM, ENCODING_WINDOWS_1252, ENCODING_ASCII), vm.settled().encodings)
        vm.onEvent(ExportDialogEvent.OnEncoding(ENCODING_ASCII))
        vm.settled { it.preview != null }
        assertEquals(true to "utf-8", exchange.asked.last().options!!.let { it.ascii to it.encoding })
        vm.onEvent(ExportDialogEvent.OnEncoding(ENCODING_WINDOWS_1252))
        vm.onEvent(ExportDialogEvent.OnFormat("csl-json"))
        val json = vm.settled { it.options.format == "csl-json" && it.preview != null }
        assertEquals(listOf(ENCODING_UTF8, ENCODING_UTF8_BOM), json.encodings)
        assertEquals(ENCODING_UTF8, json.encoding, "a .bib-only encoding falls back")
        assertNull(json.error)
    }

    @Test
    fun savedOnceOrKeptUpdated() = runTest {
        val vm = ExportDialogViewModel(ExportTarget(projectPath = "thesis"), exchange)
        vm.onEvent(ExportDialogEvent.OnSave("/tmp/refs.bib", keepUpdated = false))
        assertEquals(Notice.Done(Res.string.tools_exported, listOf(3, "/tmp/refs.bib")), vm.settled { it.done != null }.done)
        vm.onEvent(ExportDialogEvent.OnSave("/tmp/kept.bib", keepUpdated = true))
        assertEquals(Notice.Done(Res.string.tools_kept_updated, listOf(3, "/tmp/kept.bib")), vm.settled { it.done?.args?.contains("/tmp/kept.bib") == true }.done)
        assertEquals("thesis", exchange.kept.single().second.projectPath)
    }

    /** A selection is written once: what to add to it later is not known. */
    @Test
    fun aSelectionCannotBeKeptUpdated() = runTest {
        val vm = ExportDialogViewModel(ExportTarget(references = listOf("1", "2")), exchange)
        val state = vm.settled { it.preview != null }
        assertFalse(state.canKeepUpdated)
        assertEquals(listOf("1", "2"), exchange.asked.last().references)
    }
}
