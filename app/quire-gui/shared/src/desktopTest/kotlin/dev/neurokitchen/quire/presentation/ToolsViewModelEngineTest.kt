package dev.neurokitchen.quire.presentation

import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.job
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.data.engine.EngineAttachmentRepository
import dev.neurokitchen.quire.data.engine.EngineCurationRepository
import dev.neurokitchen.quire.data.engine.EngineExchangeRepository
import dev.neurokitchen.quire.data.engine.EngineLibrariesRepository
import dev.neurokitchen.quire.data.engine.EngineLibraryRepository
import dev.neurokitchen.quire.data.engine.EngineSession
import dev.neurokitchen.quire.data.engine.EngineSettingsRepository
import dev.neurokitchen.quire.data.engine.EngineSyncRepository
import dev.neurokitchen.quire.ffi.openLibraryAt
import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.presentation.screens.tools.ExportForm
import dev.neurokitchen.quire.presentation.screens.tools.ToolsConfirm
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsSection
import dev.neurokitchen.quire.presentation.screens.tools.ToolsViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.withTimeout
import java.nio.file.Files
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** The tools tab's ViewModel on a real engine, in a temporary library and config. */
@OptIn(ExperimentalCoroutinesApi::class)
class ToolsViewModelEngineTest {
    @BeforeTest
    fun setUp() = Dispatchers.setMain(Dispatchers.Default)

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun duplicatesExportAndDevicesOnTheEngine() = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val config = dir.resolve("config.toml").toString()
            val session = EngineSession(open = { openLibraryAt(dir.resolve("lib").toString(), config) })
            val exchange = EngineExchangeRepository(session)
            exchange.importBibText(
                "@article{hafting2005, title = {Microstructure of a spatial map in the entorhinal cortex}, author = {Hafting, T.}, year = {2005}}\n" +
                    "@article{hafting2005b, title = {Microstructure of a Spatial Map in the Entorhinal Cortex}, author = {Hafting, Torkel}, year = {2005}}",
            )
            val vm = ToolsViewModel(
                EngineLibraryRepository(session), EngineCurationRepository(session), exchange, EngineAttachmentRepository(session),
                EngineLibrariesRepository(session, config), EngineSyncRepository(session), EngineSettingsRepository(config),
            )
            val collecting = launch(Dispatchers.Default) { vm.state.collect {} }

            val group = vm.state.first { it.duplicates != null }.duplicates!!.single()
            vm.onEvent(ToolsEvent.OnConfirm(ToolsConfirm.Merge(group.keep, group.others.single().reference)))
            vm.onEvent(ToolsEvent.OnConfirmed)
            val merged = vm.state.first { it.notice != null }
            assertTrue(merged.notice is Notice.Done, merged.notice.toString())
            assertEquals(emptyList(), merged.duplicates)
            vm.onEvent(ToolsEvent.OnDismissNotice)

            vm.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Export))
            vm.state.first { it.formats.isNotEmpty() }
            vm.onEvent(ToolsEvent.OnExportChange(ExportForm(format = "bibtex")))
            val out = dir.resolve("out.bib")
            vm.onEvent(ToolsEvent.OnExportSave(out.toString()))
            assertTrue(vm.state.first { it.notice != null }.notice is Notice.Done)
            assertEquals(1, Regex("@article\\{").findAll(Files.readString(out)).count(), "one reference after the merge")
            vm.onEvent(ToolsEvent.OnDismissNotice)

            vm.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Sync))
            // A device ID the engine accepts (it checks the form): this device's own.
            val deviceId = vm.state.first { it.sync.deviceId != null }.sync.deviceId!!
            vm.onEvent(ToolsEvent.OnAddPeer("laptop", "not a device"))
            assertTrue(vm.state.first { it.notice != null }.notice is Notice.Failed, "the engine refuses a malformed ID")
            vm.onEvent(ToolsEvent.OnDismissNotice)
            vm.onEvent(ToolsEvent.OnAddPeer("laptop", deviceId))
            val added = vm.state.first { it.notice != null }
            assertTrue(added.sync.peers.isNotEmpty(), added.notice.toString())
            assertTrue(deviceId in Files.readString(dir.resolve("config.toml")), "devices live in this config")
            collecting.cancel()
            // As when the screen goes, and waited for: nothing may run on Main once the test resets it.
            vm.viewModelScope.coroutineContext.job.cancelAndJoin()
        }
    }
}
