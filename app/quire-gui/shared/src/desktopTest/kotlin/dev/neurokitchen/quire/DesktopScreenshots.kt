package dev.neurokitchen.quire

import dev.neurokitchen.quire.presentation.components.export_dialog.ENCODING_ASCII
import dev.neurokitchen.quire.presentation.components.export_dialog.ENCODING_WINDOWS_1252
import dev.neurokitchen.quire.presentation.components.export_dialog.ENCODING_UTF8_BOM
import dev.neurokitchen.quire.presentation.components.export_dialog.ENCODING_UTF8
import dev.neurokitchen.quire.presentation.components.export_dialog.OmitField
import dev.neurokitchen.quire.presentation.components.export_dialog.ExportTarget
import dev.neurokitchen.quire.presentation.components.export_dialog.ExportDialogContent
import dev.neurokitchen.quire.presentation.components.export_dialog.ExportDialogState
import dev.neurokitchen.quire.domain.model.ExportOptions
import dev.neurokitchen.quire.domain.model.PaperTabKind
import dev.neurokitchen.quire.domain.model.PaperTab
import dev.neurokitchen.quire.domain.model.BlockKind
import dev.neurokitchen.quire.domain.model.Word
import dev.neurokitchen.quire.domain.model.TextBlock
import dev.neurokitchen.quire.presentation.mapping.toImageBitmap
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.ImageComposeScene
import androidx.compose.ui.unit.Density
import androidx.compose.ui.use
import androidx.compose.foundation.layout.Box
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.presentation.components.commandpalette.CommandPaletteDialog
import dev.neurokitchen.quire.presentation.components.graph.CitationGraphDialog
import dev.neurokitchen.quire.presentation.desktop.OpenPaper
import dev.neurokitchen.quire.presentation.screens.library.components.QuickLookModal
import dev.neurokitchen.quire.presentation.screens.reader.components.CitationDetail
import dev.neurokitchen.quire.presentation.screens.reader.components.NotesPanel
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

    /** A table as a grid: columns as wide as their cells, the header bold, numbers aligned right. */
    @Test
    fun readerShowsATable() {
        fun row(vararg cells: String) = TextBlock(BlockKind.TableRow, cells.map { Word(it) })
        val blocks = listOf(
            TextBlock(BlockKind.Heading(2), "Temperature sensing".split(" ").map { Word(it) }),
            TextBlock(BlockKind.Paragraph, "Table 1 compares the probes measured in living cells.".split(" ").map { Word(it) }),
            row("Probe", "Sensitivity (mK Hz⁻½)", "Accuracy (mK)", "Cells (n)"),
            row("Nitrogen-vacancy centre in nanodiamond", "9 ± 2", "44", "12"),
            row("Gold nanoparticle", "—", "100", "8"),
            row("Fluorescent polymer thermometer", "180", "250 ± 30", "21"),
            TextBlock(BlockKind.Paragraph, "The nanodiamond probes were the most accurate.".split(" ").map { Word(it) }),
        )
        val state = ReaderState(
            referenceId = "t",
            title = "Nanometre-scale thermometry in a living cell",
            isLoading = false,
            mode = ReaderMode.Text,
            tabs = listOf(PaperTab(PaperTabKind.Paper, blocks)),
            pdf = PdfViewState(isLoading = false),
        )
        render("desktop-reader-table.png") {
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
                Surface { ReaderScreen(state, {}, { index, width -> pdf.render(path, index, width).toImageBitmap() }, { null }) }
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
                Surface { ReaderScreen(state, {}, { index, width -> pdf.render(path, index, width).toImageBitmap() }, { null }) }
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
                            project = "thesis",
                        ),
                        projects = listOf("thesis" to "Thesis", "thesis/chapter-2" to "Chapter 2"),
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

    @Test
    fun oledThemeWindow() {
        startKoin { modules(appModules(sampleDataModule)) }
        render("desktop-oled.png") {
            QuireTheme(appTheme = AppTheme.Oled, desktop = true) {
                DesktopApp()
            }
        }
    }

    @Test
    fun commandPaletteOverlay() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        val state = runBlocking { vm.state.first { !it.isLoading } }
        val refs = runBlocking { SampleLibraryRepository().references(LibraryQuery()).first() }
        render("desktop-command-palette.png") {
            QuireTheme(appTheme = AppTheme.Obsidian, desktop = true) {
                Surface {
                    Box(Modifier.fillMaxSize()) {
                        LibraryScreen(state, onEvent = {})
                        CommandPaletteDialog(
                            isOpen = true,
                            onDismissRequest = {},
                            papers = listOf(OpenPaper("1", "Microstructure of a spatial map in the entorhinal cortex")),
                            allReferences = refs,
                            currentTheme = AppTheme.Obsidian,
                            onOpenPaper = { _, _ -> },
                            onSelectTab = {},
                            onSetTheme = {},
                            onOpenSettings = {},
                            onReload = {},
                            onOpenTools = {},
                            onOpenCitationGraph = {},
                        )
                    }
                }
            }
        }
    }

    @Test
    fun quickLookModal() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        vm.onEvent(LibraryEvent.OnReferenceClick("1"))
        val state = runBlocking { vm.state.first { it.selected != null } }
        render("desktop-quick-look.png") {
            QuireTheme(appTheme = AppTheme.Obsidian, desktop = true) {
                Surface {
                    Box(Modifier.fillMaxSize()) {
                        LibraryScreen(state, onEvent = {})
                        QuickLookModal(
                            reference = state.selected!!,
                            onDismissRequest = {},
                            onOpenReader = {},
                        )
                    }
                }
            }
        }
    }

    @Test
    fun splitReaderMode() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val reader = SampleReaderRepository()
        runBlocking { reader.addHighlight("3", "form a map of space", "Together they ", "the key claim") }
        val pdf = SamplePdfRepository()
        val path = "mock://papers/place-cells.pdf"
        val vm = ReaderViewModel("3", "The hippocampus as a spatial map", reader, pdf)
        vm.onEvent(ReaderEvent.OnModeChange(ReaderMode.Split))
        val state = runBlocking { vm.state.first { !it.isLoading && it.mode == ReaderMode.Split } }
        render("desktop-reader-split.png") {
            QuireTheme(darkTheme = false, desktop = true) {
                Surface {
                    ReaderScreen(state, {}, { index, width -> pdf.render(path, index, width).toImageBitmap() }, { null })
                }
            }
        }
    }

    @Test
    fun readerWithFigureDrawer() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val pdf = SamplePdfRepository()
        val path = "mock://papers/place-cells.pdf"
        val state = ReaderState(
            referenceId = "3",
            title = "The hippocampus as a spatial map",
            isLoading = false,
            mode = ReaderMode.Split,
            isFigureDrawerOpen = true,
            pdf = runBlocking { PdfViewState(path = path, pages = pdf.pageSizes(path)) },
        )
        render("desktop-reader-figures.png") {
            QuireTheme(darkTheme = true, desktop = true) {
                Surface {
                    ReaderScreen(state, {}, { index, width -> pdf.render(path, index, width).toImageBitmap() }, { null })
                }
            }
        }
    }

    @Test
    fun readerWithCitationHoverCard() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val citation = CitationDetail(
            key = "hafting2005",
            title = "Microstructure of a spatial map in the entorhinal cortex",
            authors = "Hafting, T., Fyhn, M., Molden, S., Moser, M. B., & Moser, E. I.",
            year = "2005",
            venue = "Nature, 436(7052), 801-806",
            abstract = "Topographical maps of the external world are found throughout the primary sensory and motor areas of the cortex. Grid cells in the medial entorhinal cortex are part of a universal spatial metric system.",
        )
        val state = ReaderState(
            referenceId = "3",
            title = "The hippocampus as a spatial map",
            isLoading = false,
            mode = ReaderMode.Text,
            activeCitation = citation,
        )
        render("desktop-reader-citation-card.png") {
            QuireTheme(darkTheme = true, desktop = true) {
                Surface {
                    ReaderScreen(state, {}, { _, _ -> null }, { null })
                }
            }
        }
    }

    @Test
    fun citationNetworkGraphDialog() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        val state = runBlocking { vm.state.first { !it.isLoading } }
        val refs = runBlocking { SampleLibraryRepository().references(LibraryQuery()).first() }
        render("desktop-citation-graph.png") {
            QuireTheme(appTheme = AppTheme.Obsidian, desktop = true) {
                Surface {
                    Box(Modifier.fillMaxSize()) {
                        LibraryScreen(state, onEvent = {})
                        CitationGraphDialog(
                            isOpen = true,
                            onDismissRequest = {},
                            references = refs,
                            onOpenPaper = { _, _ -> },
                        )
                    }
                }
            }
        }
    }

    @Test
    fun notesWithWikilinks() {
        val notes = listOf(
            Note("n1", "Key finding: grid cells directly connect to [[hafting2005]] and support spatial orientation according to [[moser2008]].", "Today"),
            Note("n2", "Compare with hippocampal place fields described in [[okeefe1971]].", "Yesterday"),
        )
        render("desktop-notes-wikilinks.png", width = 360, height = 600) {
            QuireTheme(appTheme = AppTheme.Obsidian, desktop = true) {
                Surface {
                    NotesPanel(
                        notes = notes,
                        draft = "Follow up on [[hafting2005]] methodology",
                        onDraftChange = {},
                        onAdd = {},
                        onRemove = {},
                        onCollapse = {},
                        onWikilinkClick = {},
                    )
                }
            }
        }
    }

    @Test
    fun itemPaneWithBacklinks() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val vm = LibraryViewModel(SampleLibraryRepository(), SampleReaderRepository(), object : ExchangeRepository {}, object : AttachmentRepository {}, object : SearchRepository {})
        vm.onEvent(LibraryEvent.OnReferenceClick("1"))
        val state = runBlocking { vm.state.first { it.selected != null } }
        render("desktop-item-backlinks.png", width = 380, height = 900) {
            QuireTheme(appTheme = AppTheme.Obsidian, desktop = true) {
                Surface {
                    ItemPane(
                        reference = state.selected,
                        itemCount = state.references.size,
                        collapsed = emptySet(),
                        onSectionToggle = {},
                        editing = null,
                        tagDraft = "",
                        noteDraft = "",
                        modifier = Modifier.fillMaxSize(),
                    )
                }
            }
        }
    }

    @Composable
    private fun Library(state: LibraryState, dark: Boolean) {
        QuireTheme(appTheme = if (dark) AppTheme.Obsidian else AppTheme.PaperLight, desktop = true) { Surface { LibraryScreen(state, onEvent = {}) } }
    }

    @Test
    fun exportDialog() {
        val state = ExportDialogState(
            target = ExportTarget(projectPath = "thesis"),
            formats = listOf("biblatex", "bibtex", "csl-json", "hayagriva"),
            encodings = listOf(ENCODING_UTF8, ENCODING_UTF8_BOM, ENCODING_WINDOWS_1252, ENCODING_ASCII),
            options = ExportOptions(format = "bibtex", monthMacros = true),
            encoding = ENCODING_WINDOWS_1252,
            omitted = setOf(OmitField.Abstract, OmitField.File),
            preview = """
                @article{hafting2005,
                  author = {Hafting, Torkel and Fyhn, Marianne and Molden, Sturla and Moser, May-Britt and Moser, Edvard I.},
                  title = {Microstructure of a spatial map in the entorhinal cortex},
                  journal = {Nature},
                  year = {2005},
                  month = aug,
                  volume = {436},
                  pages = {801--806},
                  doi = {10.1038/nature03721},
                }

                @article{okeefe1971,
                  author = {O'Keefe, J. and Dostrovsky, J.},
                  title = {The hippocampus as a spatial map},
                  journal = {Brain Research},
                  year = {1971},
                }
            """.trimIndent(),
            count = 2,
        )
        render("desktop-export.png", width = 1000, height = 700) {
            QuireTheme(darkTheme = false, desktop = true) {
                Surface {
                    ExportDialogContent(state, projects = listOf("thesis" to "Thesis", "thesis/chapter-2" to "Chapter 2"), onEvent = {}, onDismiss = {})
                }
            }
        }
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

        findRepoAssetsGui()?.let { dir ->
            val assetFile = File(dir, name).apply { parentFile.mkdirs(); writeBytes(png) }
            assertTrue(assetFile.length() > 0)
        }
    }

    private fun findRepoAssetsGui(): File? {
        var cur: File? = File(".").canonicalFile
        while (cur != null) {
            val candidate = File(cur, "assets")
            if (candidate.exists() && candidate.isDirectory) {
                return File(candidate, "gui").apply { mkdirs() }
            }
            cur = cur.parentFile
        }
        return null
    }
}
