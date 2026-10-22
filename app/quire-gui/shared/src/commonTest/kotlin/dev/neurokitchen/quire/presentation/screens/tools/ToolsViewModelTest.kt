package dev.neurokitchen.quire.presentation.screens.tools

import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.data.repository.SampleSettingsRepository
import dev.neurokitchen.quire.domain.model.Duplicate
import dev.neurokitchen.quire.domain.model.HubDevice
import dev.neurokitchen.quire.domain.model.HubLink
import dev.neurokitchen.quire.domain.model.LinkJoin
import dev.neurokitchen.quire.domain.model.LinkOffer
import dev.neurokitchen.quire.domain.model.LinkRequest
import dev.neurokitchen.quire.domain.model.HubReport
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
        val autos = mutableListOf<dev.neurokitchen.quire.domain.model.AutoExport>()
        override suspend fun autoExports() = autos.toList()
    }

    private class FakeSync : SyncRepository {
        var userId = "user-1"
        override suspend fun deviceId() = "device-1"
        override suspend fun userId() = userId
        val linking = mutableListOf<String>()
        override suspend fun linkOffer() = object : LinkOffer {
            override val uri = "quire://device?id=offer"
            override suspend fun nextRequest() = LinkRequest("Pixel 9", listOf("dune", "maple", "mint"))
            override suspend fun allow() {
                linking += "allow"
            }
            override suspend fun refuse() {
                linking += "refuse"
            }
            override suspend fun cancel() {
                linking += "cancel"
            }
        }
        override suspend fun linkJoin(code: String, name: String) = object : LinkJoin {
            override val words = listOf("dune", "maple", "mint")
            override suspend fun finish() = "user-2".also {
                userId = it
                linking += "joined $code as $name"
            }
            override suspend fun cancel() {
                linking += "cancel"
            }
        }
        override suspend fun syncWith(peer: String) = SyncReport(peer, null, false, 3, 2, 0, 0, 0, 0)
        val codes = mutableListOf<String?>()
        val revoked = mutableListOf<String>()
        override suspend fun syncWithHub(code: String?) = HubReport(code != null, false, 4, 1, 2, 0, 0, 0, 0).also { codes += code }
        override suspend fun hubDevices() = listOf(HubDevice("device-1", 0, 0, false), HubDevice("phone", 0, 0, "phone" in revoked))
        override suspend fun parseHubLink(text: String) =
            if (text.startsWith("quire://hub")) HubLink("hub-2", "http://10.0.0.2:3340", "ABCD-EFGH") else error("not a quire hub link")
        override suspend fun revokeHubDevice(device: String) {
            revoked += device
        }
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

    /** The dialog writes the file; Tools closes it, reports it, and lists the file if it is kept updated. */
    @Test
    fun theExportDialogOpensAndItsSaveIsReported() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Export))
        vm.onEvent(ToolsEvent.OnShowExport(true))
        assertTrue(vm.state.first { it.isExportOpen }.isExportOpen)
        exchange.autos += dev.neurokitchen.quire.domain.model.AutoExport("/tmp/refs.bib", null, true, null, null, "2026-10-21")
        val done = Notice.Done(Res.string.tools_exported, listOf(3, "/tmp/refs.bib"))
        vm.onEvent(ToolsEvent.OnExportDone(done))
        val state = vm.state.first { !it.isExportOpen && it.autoExports.isNotEmpty() }
        assertEquals(done, state.notice)
        assertEquals(listOf("/tmp/refs.bib"), state.autoExports.map { it.path })
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

    /** A device asking with the right code is shown with its words; nothing is given until allowed. */
    @Test
    fun aLinkGivesOnlyWhenAllowed() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnLinkDevice)
        val asking = vm.state.first { it.sync.linking is LinkingState.Asking }.sync.linking as LinkingState.Asking
        assertEquals("Pixel 9" to listOf("dune", "maple", "mint"), asking.name to asking.words)
        assertTrue(sync.linking.isEmpty(), "nothing given before the person allows")
        vm.onEvent(ToolsEvent.OnAllowLink)
        assertEquals(listOf("allow"), sync.linking)
        assertNull(vm.state.first { it.notice != null }.sync.linking)
    }

    @Test
    fun aRefusedLinkGivesNothing() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnLinkDevice)
        vm.state.first { it.sync.linking is LinkingState.Asking }
        vm.onEvent(ToolsEvent.OnRefuseLink)
        assertEquals(listOf("refuse"), sync.linking)
    }

    /** Joining takes the other device's identity and reopens the library (its settings changed). */
    @Test
    fun joiningTakesTheIdentityAndReopens() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Sync))
        vm.state.first { it.sync.userId == "user-1" }
        vm.onEvent(ToolsEvent.OnJoinLink(" quire://device?id=offer ", "Laptop"))
        val state = vm.state.first { it.sync.userId == "user-2" }
        assertNull(state.sync.linking)
        assertEquals(listOf("joined quire://device?id=offer as Laptop"), sync.linking)
    }

    /** The hub's ID and relay are settings (a blank relay unsets it); the code goes with the first sync only. */
    @Test
    fun theHubIsSettingsAndSyncsWithTheCodeOnce() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Sync))
        vm.state.first { it.sync.deviceId != null }
        vm.onEvent(ToolsEvent.OnSaveHub(" hub-1 ", " http://10.0.0.2:3340 "))
        val saved = vm.state.first { it.sync.hubId != null }.sync
        assertEquals("hub-1" to "http://10.0.0.2:3340", saved.hubId to saved.relay)
        vm.onEvent(ToolsEvent.OnSyncHub(" ABCD-EFGH "))
        assertTrue(vm.state.first { it.sync.hubReport != null }.sync.hubReport!!.registered)
        vm.onEvent(ToolsEvent.OnSyncHub(""))
        assertEquals(listOf("ABCD-EFGH", null), sync.codes)
        vm.onEvent(ToolsEvent.OnSaveHub("hub-1", ""))
        vm.state.first { it.sync.relay == null }
        assertTrue(settings.entries().none { it.key == "sync.relay" })
    }

    /** A link sets the hub and relay and syncs with its code, in one step; a bad one changes nothing. */
    @Test
    fun aHubLinkJoinsInOneStep() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnUseHubLink("https://example.com"))
        assertTrue(vm.state.first { it.notice != null }.notice is Notice.Failed)
        assertTrue(settings.entries().none { it.key == "hub.id" })
        vm.onEvent(ToolsEvent.OnUseHubLink("quire://hub?id=hub-2"))
        val synced = vm.state.first { it.sync.hubReport != null }.sync
        assertEquals("hub-2" to "http://10.0.0.2:3340", synced.hubId to synced.relay)
        assertEquals(listOf<String?>("ABCD-EFGH"), sync.codes)
    }

    @Test
    fun aHubDeviceIsRevokedOnlyOnceConfirmed() = runTest {
        val vm = viewModel()
        vm.onEvent(ToolsEvent.OnLoadHubDevices)
        assertEquals(2, vm.state.first { it.sync.hubDevices != null }.sync.hubDevices!!.size)
        vm.onEvent(ToolsEvent.OnConfirm(ToolsConfirm.RevokeHubDevice("phone")))
        assertTrue(sync.revoked.isEmpty())
        vm.onEvent(ToolsEvent.OnConfirmed)
        assertTrue(vm.state.first { s -> s.sync.hubDevices.orEmpty().any { it.revoked } }.sync.hubDevices!!.single { it.revoked }.device == "phone")
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
