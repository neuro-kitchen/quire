package dev.neurokitchen.quire

import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.ImageComposeScene
import androidx.compose.ui.unit.Density
import androidx.compose.ui.use
import dev.neurokitchen.quire.data.repository.SampleLibraryRepository
import dev.neurokitchen.quire.data.engine.EnginePdfRepository
import dev.neurokitchen.quire.data.repository.SamplePdfRepository
import dev.neurokitchen.quire.data.repository.SampleReaderRepository
import dev.neurokitchen.quire.data.repository.SampleSettingsRepository
import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.presentation.screens.reader.PdfViewState
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
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryScreen
import dev.neurokitchen.quire.presentation.screens.library.LibraryState
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
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
        val vm = LibraryViewModel(SampleLibraryRepository())
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
    fun readerOnAPdf() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        val path = java.nio.file.Files.createTempFile("paper", ".pdf").also {
            java.nio.file.Files.write(it, tinyPdf(listOf("Place cells in the hippocampus fire", "when the animal is in one place.")))
        }.toString()
        val pdf = EnginePdfRepository { System.getProperty("quire.pdfium.dir") }
        val highlight = Highlight("h1", "hippocampus fire", comment = "first place cells")
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

    @Composable
    private fun Library(state: LibraryState, dark: Boolean) {
        QuireTheme(darkTheme = dark, desktop = true) { Surface { LibraryScreen(state, onEvent = {}) } }
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
