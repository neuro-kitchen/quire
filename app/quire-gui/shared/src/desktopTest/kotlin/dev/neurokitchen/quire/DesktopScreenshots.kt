package dev.neurokitchen.quire

import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.ImageComposeScene
import androidx.compose.ui.unit.Density
import androidx.compose.ui.use
import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.presentation.screens.library.components.ItemPane
import dev.neurokitchen.quire.presentation.screens.library.components.NameDialogView
import dev.neurokitchen.quire.presentation.screens.library.components.ProjectsPane
import dev.neurokitchen.quire.presentation.screens.library.components.ReferenceDrag
import dev.neurokitchen.quire.presentation.screens.library.NameDialog
import dev.neurokitchen.quire.presentation.screens.reader.FindState
import dev.neurokitchen.quire.data.engine.EnginePdfRepository
import dev.neurokitchen.quire.presentation.screens.tools.DuplicateGroupState
import dev.neurokitchen.quire.presentation.screens.tools.DuplicateState
import dev.neurokitchen.quire.presentation.screens.tools.PeerState
import dev.neurokitchen.quire.presentation.screens.tools.ProjectChoice
import dev.neurokitchen.quire.presentation.screens.tools.ReferenceLine
import dev.neurokitchen.quire.presentation.screens.tools.SyncState
import dev.neurokitchen.quire.presentation.screens.tools.ToolsScreen
import dev.neurokitchen.quire.presentation.screens.tools.ToolsSection
import dev.neurokitchen.quire.presentation.screens.tools.ToolsState
import dev.neurokitchen.quire.presentation.screens.library.SavedSearchState
import androidx.compose.ui.geometry.Offset
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.Modifier
import dev.neurokitchen.quire.domain.repository.AttachmentRepository
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.domain.repository.SearchRepository
import dev.neurokitchen.quire.data.repository.SamplePdfRepository
import dev.neurokitchen.quire.data.repository.SampleReaderRepository
import dev.neurokitchen.quire.data.repository.SampleSettingsRepository
import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.presentation.screens.reader.PdfViewState
import dev.neurokitchen.quire.presentation.screens.reader.ReaderEvent
import dev.neurokitchen.quire.presentation.screens.reader.ReaderMode
import dev.neurokitchen.quire.presentation.screens.reader.ReaderScreen
import dev.neurokitchen.quire.presentation.screens.reader.ReaderState
import dev.neurokitchen.quire.presentation.screens.reader.ReaderViewModel
import dev.neurokitchen.quire.presentation.screens.settings.SettingsEvent
import dev.neurokitchen.quire.presentation.screens.settings.SettingsScreen
import dev.neurokitchen.quire.presentation.screens.settings.SettingsSection
import dev.neurokitchen.quire.presentation.screens.settings.SettingsViewModel
import dev.neurokitchen.quire.di.appModules
import dev.neurokitchen.quire.di.sampleDataModule
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.presentation.desktop.DesktopApp
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryScreen
import dev.neurokitchen.quire.presentation.screens.library.LibraryState
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.components.reference_card.ReferenceCardState
import dev.neurokitchen.quire.presentation.screens.library.AddDialogState
import dev.neurokitchen.quire.presentation.screens.library.components.AddReferenceDialog
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.jetbrains.skia.EncodedImageFormat
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import java.io.File
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertTrue

/** Renders desktop windows offscreen to `build/screenshots/` to look at them, light and dark. */
@OptIn(ExperimentalCoroutinesApi::class)
class DesktopScreenshots {
    @AfterTest
    fun tearDown() {
        stopKoin()
        Dispatchers.resetMain()
    }

    @Test
    fun mainWindow() {
        startKoin { modules(appModules(sampleDataModule)) }
        render("desktop-main.png") { DesktopApp() }
    }

    @Test
    fun selectedReferenceInAProject() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        listOf(
            LibraryEvent.OnProjectExpandToggle("thesis"),
            LibraryEvent.OnSourceClick(LibrarySource.InProject("thesis")),
            LibraryEvent.OnReferenceClick("1"),
        ).forEach(vm::onEvent)
        val state = runBlocking { vm.state.first { it.selected != null } }
        for (dark in listOf(false, true)) {
            render("desktop-selected-${if (dark) "dark" else "light"}.png") { Library(state, dark) }
        }
    }

    /** The item pane alone and tall, so every section shows; then the Info section being edited. */
    @Test
    fun itemPaneSections() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        vm.onEvent(LibraryEvent.OnReferenceClick("3"))
        vm.onEvent(LibraryEvent.OnTagDraftChange("grid cells"))
        val state = runBlocking { vm.state.first { it.selected != null } }
        render("desktop-item-pane.png", width = 380, height = 1500) { Pane(state) }
        vm.onEvent(LibraryEvent.OnEditStart)
        val editing = runBlocking { vm.state.first { it.editing != null } }
        render("desktop-item-edit.png", width = 380, height = 900) { Pane(editing) }
    }

    @Composable
    private fun Pane(state: LibraryState) {
        QuireTheme(desktop = true) {
            Surface {
                ItemPane(
                    reference = state.selected, itemCount = state.references.size, collapsed = emptySet(), onSectionToggle = {},
                    editing = state.editing, tagDraft = state.tagDraft, noteDraft = state.noteDraft, modifier = Modifier.fillMaxSize(),
                )
            }
        }
    }

    @Test
    fun settingsWindow() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = SettingsViewModel(SampleSettingsRepository(), SampleLibraryRepository())
        vm.onEvent(SettingsEvent.OnSectionClick(SettingsSection.Reader))
        val state = runBlocking { vm.state.first { !it.isLoading } }
        render("desktop-settings.png", width = 860, height = 600) {
            QuireTheme(darkTheme = false, desktop = true) { Surface { SettingsScreen(state, onEvent = {}) } }
        }
    }

    @Test
    fun readerOnTheText() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val reader = SampleReaderRepository()
        runBlocking { reader.addHighlight("3", "form a map of space", "Together they ", "the key claim") }
        val vm = ReaderViewModel("3", "The hippocampus as a spatial map", reader, SamplePdfRepository())
        val state = runBlocking { vm.state.first { !it.isLoading && it.highlights.isNotEmpty() } }
        render("desktop-reader-text.png") {
            QuireTheme(darkTheme = false, desktop = true) { Surface { ReaderScreen(state, {}, { _, _ -> null }, { null }) } }
        }
    }

    @Test
    fun readerThemedAndCustomFont() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val reader = SampleReaderRepository()
        runBlocking { reader.addHighlight("3", "form a map of space", "Together they ", "the key claim") }
        val vm = ReaderViewModel("3", "The hippocampus as a spatial map", reader, SamplePdfRepository())
        vm.onEvent(ReaderEvent.OnSetReaderTheme(dev.neurokitchen.quire.presentation.designsystem.ReaderTheme.Sepia))
        vm.onEvent(ReaderEvent.OnSetReaderFont(dev.neurokitchen.quire.presentation.designsystem.ReaderFont.Serif))
        vm.onEvent(ReaderEvent.OnChangeFontSize(2))
        val state = runBlocking { vm.state.first { !it.isLoading && it.highlights.isNotEmpty() } }
        render("desktop-reader-sepia.png") {
            QuireTheme(darkTheme = false, desktop = true) { Surface { ReaderScreen(state, {}, { _, _ -> null }, { null }) } }
        }
    }

    @Test
    fun libraryWithCollapsedActivityRail() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        vm.onEvent(LibraryEvent.OnToggleSidebar)
        val state = runBlocking { vm.state.first { it.isSidebarCollapsed } }
        render("desktop-activity-rail.png") {
            QuireTheme(darkTheme = true, desktop = true) { Surface { LibraryScreen(state, onEvent = {}) } }
        }
    }

    @Test
    fun libraryDashboardOverview() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        val state = runBlocking { vm.state.first { !it.isLoading && it.selected == null } }
        render("desktop-dashboard.png") {
            QuireTheme(darkTheme = true, desktop = true) { Surface { LibraryScreen(state, onEvent = {}) } }
        }
    }

    @Test
    fun readerOnAPdf() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val pdf = SamplePdfRepository()
        val path = "mock://papers/place-cells.pdf"
        val highlight = Highlight("h1", "hippocampus fire", comment = "first place cells", color = "yellow")
        val state = runBlocking {
            ReaderState(
                referenceId = "3", title = "Place cells", isLoading = false, mode = ReaderMode.Pdf,
                pdf = PdfViewState(path = path, pages = pdf.pageSizes(path), locations = pdf.locate(path, listOf(highlight))),
                highlights = listOf(highlight),
            )
        }
        render("desktop-reader-pdf.png") {
            QuireTheme(darkTheme = false, desktop = true) {
                Surface { ReaderScreen(state, {}, { index, width -> pdf.render(path, index, width) }, { null }) }
            }
        }
    }

    /** A real PDF through the engine with the find bar open: both matches drawn, the second current. */
    @Test
    fun readerFindOnAPdf() {
        val file = java.nio.file.Files.createTempFile("paper", ".pdf")
        java.nio.file.Files.write(file, tinyPdf(listOf("Place cells fire in one place", "and grid cells tile the floor")))
        val path = file.toString()
        val pdf = EnginePdfRepository()
        val state = runBlocking {
            ReaderState(
                referenceId = "3", title = "Place cells", isLoading = false, mode = ReaderMode.Pdf,
                pdf = PdfViewState(path = path, pages = pdf.pageSizes(path)),
                find = FindState(isOpen = true, query = "place", pdfMatches = pdf.search(path, "place"), current = 1),
            )
        }
        render("desktop-reader-find.png", width = 1100, height = 760) {
            QuireTheme(desktop = true) {
                Surface { ReaderScreen(state, {}, { index, width -> pdf.render(path, index, width) }, { null }) }
            }
        }
    }

    /** Two references dragged over a project (highlighted), a coloured tag; then renaming a project. */
    @Test
    fun projectsPaneDropAndRename() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        vm.onEvent(LibraryEvent.OnProjectExpandToggle("thesis"))
        val state = runBlocking { vm.state.first { it.projects.size == 4 && it.tags.isNotEmpty() } }
        // Rows are 28dp tall below the 41dp toolbar: My Library, Deep learning, Reading group, Thesis.
        val drag = ReferenceDrag().apply { start(setOf("1", "2"), Offset(120f, 41f + 28f * 3 + 14f)) }
        render("desktop-projects-drop.png", width = 260, height = 560) {
            QuireTheme(desktop = true) { Surface { ProjectsPane(state, onEvent = {}, drag = drag) } }
        }
        render("desktop-rename-project.png", width = 600, height = 360) {
            QuireTheme(desktop = true) {
                Surface { NameDialogView(NameDialog.RenameProject("thesis", "Thesis"), onDismiss = {}, onConfirm = {}) }
            }
        }
    }

    /** A search the engine refuses as typed (the last results stay), with a saved search in the pane. */
    @Test
    fun searchErrorAndSavedSearch() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        vm.onEvent(LibraryEvent.OnQueryChange("neuro"))
        val state = runBlocking { vm.state.first { it.query == "neuro" } }.copy(
            query = "neuro status:r",
            searchError = "invalid filter status:r: unread, reading or read",
            savedSearches = listOf(SavedSearchState("Unread neuroscience", "neuro status:unread", isActive = false)),
        )
        render("desktop-search-error.png") { Library(state, dark = false) }
    }

    /** The tools tab: a duplicate to merge, then this device and its peers. */
    @Test
    fun toolsTab() {
        val keep = ReferenceLine("1", "Microstructure of a spatial map in the entorhinal cortex", "Hafting et al. · 2005", "hafting2005")
        val other = ReferenceLine("2", "Microstructure of a Spatial Map in the Entorhinal Cortex", "Hafting · 2005", "hafting2005b")
        val duplicates = ToolsState(duplicates = listOf(DuplicateGroupState(keep, listOf(DuplicateState(other, null)))))
        render("desktop-tools-duplicates.png", width = 1100, height = 520) {
            QuireTheme(desktop = true) { Surface { ToolsScreen(duplicates, onEvent = {}) } }
        }
        val sync = ToolsState(
            section = ToolsSection.Sync,
            projects = listOf(ProjectChoice("thesis", "Thesis")),
            sync = SyncState(
                deviceId = "k3q7v2m9x4c8w1p5n6r0t2y7u3i9o4e1a8s5d2f6g0h3j7l1z9b4",
                userId = "u5n8e2w7q1r4t9y3",
                peers = listOf(PeerState("laptop", "p9o8i7u6y5t4r3e2w1q0a9s8d7f6g5h4j3k2l1z0x9c8v7b6n5m4")),
            ),
        )
        render("desktop-tools-sync.png", width = 1100, height = 720) {
            QuireTheme(desktop = true) { Surface { ToolsScreen(sync, onEvent = {}) } }
        }
    }

    @Test
    fun addReferenceDialog() {
        render("desktop-add-reference.png", width = 800, height = 650) {
            QuireTheme(darkTheme = false, desktop = true) {
                Surface {
                    AddReferenceDialog(
                        state = AddDialogState(
                            identifier = "10.1038/nature03721",
                            preview = ReferenceCardState(
                                id = "", citeKey = "", kind = "article-journal",
                                title = "Microstructure of a spatial map in the entorhinal cortex",
                                creator = "Hafting et al.", year = "2005", dateAdded = "", hasPdf = false,
                            ),
                        ),
                        onEvent = {},
                    )
                }
            }
        }
    }

    @Test
    fun nordThemeWindow() {
        startKoin { modules(appModules(sampleDataModule)) }
        render("desktop-nord.png") {
            QuireTheme(appTheme = dev.neurokitchen.quire.presentation.designsystem.AppTheme.Nord, desktop = true) {
                DesktopApp()
            }
        }
    }

    @Test
    fun solarizedThemeWindow() {
        startKoin { modules(appModules(sampleDataModule)) }
        render("desktop-solarized.png") {
            QuireTheme(appTheme = dev.neurokitchen.quire.presentation.designsystem.AppTheme.Solarized, desktop = true) {
                DesktopApp()
            }
        }
    }

    @Test
    fun multiSelectionTable() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        vm.onEvent(LibraryEvent.OnReferenceSelect("1", isCtrlPressed = false, isShiftPressed = false))
        vm.onEvent(LibraryEvent.OnReferenceSelect("2", isCtrlPressed = true, isShiftPressed = false))
        vm.onEvent(LibraryEvent.OnReferenceSelect("3", isCtrlPressed = true, isShiftPressed = false))
        val state = runBlocking { vm.state.first { it.selectedIds.size >= 3 } }
        render("desktop-multi-select.png") {
            QuireTheme(appTheme = dev.neurokitchen.quire.presentation.designsystem.AppTheme.Obsidian, desktop = true) {
                Surface { LibraryScreen(state, onEvent = {}) }
            }
        }
    }

    @Test
    fun settingsAppearanceCards() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = SettingsViewModel(SampleSettingsRepository(), SampleLibraryRepository())
        vm.onEvent(SettingsEvent.OnSectionClick(SettingsSection.Appearance))
        val state = runBlocking { vm.state.first { it.section == SettingsSection.Appearance } }
        render("desktop-settings-appearance.png", width = 900, height = 650) {
            QuireTheme(appTheme = dev.neurokitchen.quire.presentation.designsystem.AppTheme.Obsidian, desktop = true) {
                Surface { SettingsScreen(state, onEvent = {}) }
            }
        }
    }

    @Test
    fun settingsShortcutsCheatSheet() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = SettingsViewModel(SampleSettingsRepository(), SampleLibraryRepository())
        vm.onEvent(SettingsEvent.OnSectionClick(SettingsSection.Shortcuts))
        val state = runBlocking { vm.state.first { it.section == SettingsSection.Shortcuts } }
        render("desktop-settings-shortcuts.png", width = 900, height = 650) {
            QuireTheme(appTheme = dev.neurokitchen.quire.presentation.designsystem.AppTheme.Obsidian, desktop = true) {
                Surface { SettingsScreen(state, onEvent = {}) }
            }
        }
    }

    @Composable
    private fun Library(state: LibraryState, dark: Boolean) {
        QuireTheme(appTheme = if (dark) AppTheme.Obsidian else AppTheme.PaperLight, desktop = true) { Surface { LibraryScreen(state, onEvent = {}) } }
    }

    private fun render(name: String, width: Int = 1400, height: Int = 860, content: @Composable () -> Unit) {
        val png = ImageComposeScene(width, height, Density(1f), content = content).use { scene ->
            // The first frames start loading resources and pages; later ones draw them.
            scene.render()
            Thread.sleep(300)
            scene.render(500_000_000L)
            Thread.sleep(300)
            scene.render(1_000_000_000L).encodeToData(EncodedImageFormat.PNG)!!.bytes
        }
        val file = File("build/screenshots/$name").apply { parentFile.mkdirs(); writeBytes(png) }
        assertTrue(file.length() > 0)
    }
}
