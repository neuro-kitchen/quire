package dev.neurokitchen.quire.presentation

import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.job
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.presentation.components.notice.Notice
import dev.neurokitchen.quire.data.engine.EngineAttachmentRepository
import dev.neurokitchen.quire.data.engine.EngineExchangeRepository
import dev.neurokitchen.quire.data.engine.EngineLibraryRepository
import dev.neurokitchen.quire.data.engine.EngineReaderRepository
import dev.neurokitchen.quire.data.engine.EngineSearchRepository
import dev.neurokitchen.quire.data.engine.EngineSession
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.ffi.openLibraryAt
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.presentation.screens.library.AddTab
import dev.neurokitchen.quire.presentation.screens.library.Confirm
import dev.neurokitchen.quire.presentation.screens.library.NameDialog
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.library.ManualEntry
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

/** The library screen's ViewModel on the real engine: what the user does reaches the library. */
@OptIn(ExperimentalCoroutinesApi::class)
class LibraryViewModelEngineTest {
    @BeforeTest
    fun setUp() = Dispatchers.setMain(Dispatchers.Default)

    @AfterTest
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun addingThroughTheDialogReachesTheLibrary() = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val session = EngineSession(open = { openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString()) })
            val library = EngineLibraryRepository(session)
            val vm = LibraryViewModel(library, EngineReaderRepository(session), EngineExchangeRepository(session), EngineAttachmentRepository(session), EngineSearchRepository(session))
            val collecting = launch(Dispatchers.Default) { vm.state.collect {} }

            vm.onEvent(LibraryEvent.OnShowAddDialog(true))
            vm.onEvent(LibraryEvent.OnAddTabSelect(AddTab.Manual))
            vm.onEvent(LibraryEvent.OnManualChange(ManualEntry(title = "Grid cells", authors = "Hafting, Torkel", year = "2005", tags = "grid, rats")))
            vm.onEvent(LibraryEvent.OnAddConfirm)
            assertTrue(vm.state.first { it.notice != null }.notice is Notice.Done)
            val added = vm.state.first { s -> s.references.any { it.title == "Grid cells" } }.references.single()
            assertEquals("2005", added.year)
            assertEquals(listOf("grid", "rats"), library.references(LibraryQuery(text = "grid cells")).first().single().tags.sorted())

            vm.onEvent(LibraryEvent.OnDismissNotice)
            vm.onEvent(LibraryEvent.OnShowAddDialog(true))
            vm.onEvent(LibraryEvent.OnAddTabSelect(AddTab.Bibtex))
            vm.onEvent(LibraryEvent.OnBibtexChange("@book{buzsaki2006, title = {Rhythms of the Brain}, author = {Buzsáki, György}, year = {2006}}"))
            vm.onEvent(LibraryEvent.OnAddConfirm)
            assertEquals(2, vm.state.first { s -> s.references.any { it.citeKey == "buzsaki2006" } }.references.size)

            // A lookup the engine refuses: its message, in the dialog.
            vm.onEvent(LibraryEvent.OnShowAddDialog(true))
            vm.onEvent(LibraryEvent.OnIdentifierChange("not an identifier"))
            vm.onEvent(LibraryEvent.OnLookup)
            assertTrue(vm.state.first { it.addDialog?.error != null }.addDialog!!.error!!.isNotBlank())
            collecting.cancel()
            // As when the screen goes, and waited for: nothing may run on Main once the test resets it.
            vm.viewModelScope.coroutineContext.job.cancelAndJoin()
        }
    }

    /** Projects and tags managed from the projects pane reach the engine, and the pane follows. */
    @Test
    fun projectsAndTagsAreManagedOnTheEngine() = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val session = EngineSession(open = { openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString()) })
            val library = EngineLibraryRepository(session)
            val vm = LibraryViewModel(library, EngineReaderRepository(session), EngineExchangeRepository(session), EngineAttachmentRepository(session), EngineSearchRepository(session))
            val collecting = launch(Dispatchers.Default) { vm.state.collect {} }

            vm.onEvent(LibraryEvent.OnShowAddDialog(true))
            vm.onEvent(LibraryEvent.OnAddTabSelect(AddTab.Manual))
            vm.onEvent(LibraryEvent.OnManualChange(ManualEntry(title = "Grid cells", authors = "Hafting, Torkel", year = "2005", tags = "grid")))
            vm.onEvent(LibraryEvent.OnAddConfirm)
            val id = vm.state.first { it.references.isNotEmpty() }.references.single().id

            vm.onEvent(LibraryEvent.OnNameDialog(NameDialog.NewProject(withIds = setOf(id))))
            vm.onEvent(LibraryEvent.OnNameConfirm("Navigation"))
            val path = vm.state.first { it.projects.isNotEmpty() }.projects.single().id
            vm.onEvent(LibraryEvent.OnSourceClick(LibrarySource.InProject(path)))
            vm.onEvent(LibraryEvent.OnNameDialog(NameDialog.RenameProject(path, "Navigation")))
            vm.onEvent(LibraryEvent.OnNameConfirm("Space"))
            val renamed = vm.state.first { s -> s.projects.any { it.name == "Space" } }
            val source = vm.state.first { it.source != LibrarySource.InProject(path) && it.references.size == 1 }.source
            assertEquals(LibrarySource.InProject(renamed.projects.single().id), source, "the view follows the rename")

            vm.onEvent(LibraryEvent.OnTagColor("grid", "#5FB236"))
            assertEquals("#5fb236", vm.state.first { s -> s.tags.any { it.color != null } }.tags.first().color?.lowercase())

            vm.onEvent(LibraryEvent.OnConfirm(Confirm.DeleteTag("grid")))
            vm.onEvent(LibraryEvent.OnConfirmed)
            vm.state.first { it.tags.isEmpty() }
            vm.onEvent(LibraryEvent.OnConfirm(Confirm.DeleteProject(source.let { (it as LibrarySource.InProject).path }, "Space")))
            vm.onEvent(LibraryEvent.OnConfirmed)
            vm.state.first { it.projects.isEmpty() && it.source == LibrarySource.All }
            assertEquals(1, library.references(LibraryQuery()).first().size, "the reference stays in the library")
            collecting.cancel()
            // As when the screen goes, and waited for: nothing may run on Main once the test resets it.
            vm.viewModelScope.coroutineContext.job.cancelAndJoin()
        }
    }

    /** The engine's search syntax from the toolbar: a half-typed filter is reported, a search is saved and run. */
    @Test
    fun searchingOnTheEngine() = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val session = EngineSession(open = { openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString()) })
            val vm = LibraryViewModel(
                EngineLibraryRepository(session), EngineReaderRepository(session), EngineExchangeRepository(session),
                EngineAttachmentRepository(session), EngineSearchRepository(session),
            )
            val collecting = launch(Dispatchers.Default) { vm.state.collect {} }
            vm.onEvent(LibraryEvent.OnShowAddDialog(true))
            vm.onEvent(LibraryEvent.OnAddTabSelect(AddTab.Bibtex))
            vm.onEvent(
                LibraryEvent.OnBibtexChange(
                    "@article{hafting2005, title = {Microstructure of a spatial map}, author = {Hafting, T.}, year = {2005}}\n" +
                        "@book{buzsaki2006, title = {Rhythms of the Brain}, author = {Buzsáki, György}, year = {2006}}",
                ),
            )
            vm.onEvent(LibraryEvent.OnAddConfirm)
            vm.state.first { it.references.size == 2 }

            vm.onEvent(LibraryEvent.OnQueryChange("year:2005"))
            assertEquals(listOf("hafting2005"), vm.state.first { it.query == "year:2005" && it.references.size == 1 }.references.map { it.citeKey })
            vm.onEvent(LibraryEvent.OnQueryChange("status:r"))
            val refused = vm.state.first { it.searchError != null }
            assertTrue("status" in refused.searchError!!, refused.searchError)
            assertEquals(listOf("hafting2005"), refused.references.map { it.citeKey }, "the last results stay")

            vm.onEvent(LibraryEvent.OnQueryChange("rhythms"))
            vm.state.first { it.query == "rhythms" && it.searchError == null && it.references.size == 1 }
            vm.onEvent(LibraryEvent.OnNameDialog(NameDialog.SaveSearch("rhythms")))
            vm.onEvent(LibraryEvent.OnNameConfirm("Rhythms"))
            vm.state.first { it.savedSearches.isNotEmpty() }
            vm.onEvent(LibraryEvent.OnQueryChange(""))
            vm.onEvent(LibraryEvent.OnSavedSearchClick("Rhythms"))
            val running = vm.state.first { it.query == "rhythms" && it.references.size == 1 }
            assertEquals("buzsaki2006", running.references.single().citeKey)
            assertTrue(running.savedSearches.single().isActive)
            collecting.cancel()
            // As when the screen goes, and waited for: nothing may run on Main once the test resets it.
            vm.viewModelScope.coroutineContext.job.cancelAndJoin()
        }
    }
}
