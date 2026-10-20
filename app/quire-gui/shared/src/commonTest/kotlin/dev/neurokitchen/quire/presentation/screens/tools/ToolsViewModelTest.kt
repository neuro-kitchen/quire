package dev.neurokitchen.quire.presentation.screens.tools

import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.data.repository.SampleSettingsRepository
import dev.neurokitchen.quire.domain.model.Duplicate
import dev.neurokitchen.quire.domain.model.DuplicateGroup
import dev.neurokitchen.quire.domain.model.ExportSelection
import dev.neurokitchen.quire.domain.model.LibraryEntry
import dev.neurokitchen.quire.domain.model.MergeResult
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.SyncReport
import dev.neurokitchen.quire.domain.repository.AttachmentRepository
import dev.neurokitchen.quire.domain.repository.CurationRepository
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.domain.repository.LibrariesRepository
import dev.neurokitchen.quire.domain.repository.SyncRepository
import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tools_exported
import dev.neurokitchen.quire.resources.tools_merged
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
class ToolsViewModelTest {
    @BeforeTest
    fun setUp() = Dispatchers.setMain(UnconfinedTestDispatcher())

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    private fun reference(id: String, title: String) =
        Reference(id = id, citeKey = id, kind = "article-journal", title = title, creators = emptyList(), year = 2005)

    /** Two references that look alike, until merged. */
    private class FakeCuration(private val a: Reference, private val b: Reference) : CurationRepository {
        val merged = mutableListOf<Pair<String, String>>()
        override suspend fun duplicates() =
            if (merged.isEmpty()) listOf(DuplicateGroup(a, listOf(Duplicate(b, "doi:10.1038/nature03721")))) else emptyList()
        override suspend fun merge(keep: String, other: String): MergeResult {
            merged += keep to other
            return MergeResult(a, other, emptyList(), 0, 0, 0, 0, 0, 0)
        }
    }

    private class FakeExchange : ExchangeRepository {
        val written = mutableListOf<Pair<ExportSelection, String>>()
        override fun exportFormats() = listOf("biblatex", "bibtex", "csl-json", "hayagriva")
        override suspend fun export(selection: ExportSelection) = "@article{a}" to 1
        override suspend fun exportTo(selection: ExportSelection, path: String): Int {
            written += selection to path
            return 1
        }
        override suspend fun autoExports() = emptyList<dev.neurokitchen.quire.domain.model.AutoExport>()
    }

    private class FakeSync : SyncRepository {
        var userId = "user-1"
        override suspend fun deviceId() = "device-1"
        override suspend fun userId() = userId
        override suspend fun importUserKey(key: String) = "user-2".also { userId = it }
        override suspend fun syncWith(peer: String) = SyncReport(peer, null, false, 3, 2, 0, 0, 0, 0)
    }

    private class FakeLibraries : LibrariesRepository {
        val opened = mutableListOf<String?>()
        override suspend fun libraries() = listOf(LibraryEntry("main", "/libs/main", true, isDefault = true), LibraryEntry("lab", "/libs/lab", true, false))
        override suspend fun open(name: String?) {
            opened += name
        }
    }

    private val a = reference("a", "Microstructure of a spatial map")
    private val b = reference("b", "Microstructure of a spatial map in the entorhinal cortex")
    private val curation = FakeCuration(a, b)
    private val exchange = FakeExchange()
    private val sync = FakeSync()
    private val libraries = FakeLibraries()
    private val settings = SampleSettingsRepository()

    private fun viewModel() = ToolsViewModel(
        SampleLibraryRepository(), curation, exchange, object : AttachmentRepository {}, libraries, sync, settings,
    )

    @Test
    fun duplicatesShowOnOpenAndMergeOnlyOnceConfirmed() = runTest {
        val vm = viewModel()
        val group = vm.state.first { it.duplicates != null }.duplicates!!.single()
        assertEquals("a", group.keep.id)
        assertEquals("doi:10.1038/nature03721", group.others.single().sharedIdentifier)
        vm.onEvent(ToolsEvent.OnConfirm(ToolsConfirm.Merge(group.keep, group.others.single().reference)))
        assertTrue(curation.merged.isEmpty(), "nothing before confirming")
        vm.onEvent(ToolsEvent.OnConfirmed)
        val state = vm.state.first { it.notice != null }
        assertEquals(listOf("a" to "b"), curation.merged)
        assertEquals(Notice.Done(Res.string.tools_merged, listOf(a.title)), state.notice)
        assertEquals(emptyList(), state.duplicates)
        assertNull(state.confirm)
    }

    @Test
    fun theExportUsesTheForm() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Export))
        assertEquals(4, vm.state.first { it.formats.isNotEmpty() }.formats.size)
        vm.onEvent(ToolsEvent.OnExportChange(ExportForm(projectPath = "thesis", recursive = false, format = "csl-json")))
        vm.onEvent(ToolsEvent.OnExportPreview)
        assertEquals("@article{a}", vm.state.first { it.exportPreview != null }.exportPreview)
        vm.onEvent(ToolsEvent.OnExportSave("/tmp/out.json"))
        assertEquals(Notice.Done(Res.string.tools_exported, listOf(1, "/tmp/out.json")), vm.state.first { it.notice != null }.notice)
        assertEquals(listOf(ExportSelection("thesis", false, format = "csl-json") to "/tmp/out.json"), exchange.written)
    }

    @Test
    fun devicesAreSettingsAndSyncReports() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Sync))
        assertEquals("device-1", vm.state.first { it.sync.deviceId != null }.sync.deviceId)
        vm.onEvent(ToolsEvent.OnAddPeer(" laptop ", " abc123 "))
        assertEquals(listOf(PeerState("laptop", "abc123")), vm.state.first { it.sync.peers.isNotEmpty() }.sync.peers)
        assertTrue(settings.entries().any { it.key == "peers.laptop.id" && it.value == "abc123" })
        vm.onEvent(ToolsEvent.OnSyncWith("laptop"))
        assertEquals(3, vm.state.first { it.sync.lastReport != null }.sync.lastReport!!.sent)
        vm.onEvent(ToolsEvent.OnRemovePeer("laptop"))
        assertTrue(vm.state.first { it.sync.peers.isEmpty() }.sync.peers.isEmpty())
    }

    @Test
    fun anotherDevicesKeyIsUsedOnlyOnceConfirmed() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Sync))
        vm.state.first { it.sync.userId == "user-1" }
        vm.onEvent(ToolsEvent.OnConfirm(ToolsConfirm.ImportKey("secret ")))
        assertEquals("user-1", sync.userId)
        vm.onEvent(ToolsEvent.OnConfirmed)
        assertEquals("user-2", vm.state.first { it.sync.userId == "user-2" }.sync.userId)
    }

    @Test
    fun openingALibraryStartsTheSectionsAfresh() = runTest {
        val vm = viewModel()
        vm.state.first { it.duplicates != null }
        vm.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Libraries))
        assertEquals(2, vm.state.first { it.libraries.isNotEmpty() }.libraries.size)
        vm.onEvent(ToolsEvent.OnOpenLibrary("lab"))
        val state = vm.state.first { it.notice != null }
        assertEquals(listOf<String?>("lab"), libraries.opened)
        assertNull(state.duplicates, "the other library's duplicates are looked for again")
        assertEquals(ToolsSection.Libraries, state.section)
    }
}
