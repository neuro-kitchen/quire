package dev.neurokitchen.quire

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.ImageComposeScene
import androidx.compose.ui.use
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import dev.chrisbanes.haze.rememberHazeState
import dev.neurokitchen.quire.data.repository.SamplePdfRepository
import dev.neurokitchen.quire.data.repository.SampleReaderRepository
import dev.neurokitchen.quire.di.appModules
import dev.neurokitchen.quire.di.sampleDataModule
import dev.neurokitchen.quire.presentation.designsystem.AppTheme
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.mapping.toImageBitmap
import dev.neurokitchen.quire.presentation.mobile.MobileApp
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileChrome
import dev.neurokitchen.quire.presentation.mobile.navigation.MobileSettingsPage
import dev.neurokitchen.quire.presentation.mobile.screens.DetailsScreen
import dev.neurokitchen.quire.presentation.mobile.screens.LibraryTab
import dev.neurokitchen.quire.presentation.mobile.screens.SettingsPageScreen
import dev.neurokitchen.quire.presentation.mobile.screens.SettingsTab
import dev.neurokitchen.quire.presentation.screens.reader.ReaderViewModel
import dev.neurokitchen.quire.presentation.screens.reader.components.MobileReaderView
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.withTimeout
import org.jetbrains.skia.EncodedImageFormat
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import java.io.File
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * The phone app's screens rendered offscreen at a Pixel's size (412 × 915 dp at 2.625), on the
 * sample data, to `build/screenshots/phone-*.png` and `assets/gui/`. Regressions in phone layouts
 * (wrapped tabs, content under the bars) show here without an emulator.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class PhoneScreenshots {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        startKoin { modules(appModules(sampleDataModule)) }
    }

    @AfterTest
    fun tearDown() {
        stopKoin()
        Dispatchers.resetMain()
    }

    @Test
    fun home() = render("phone-home.png") { MobileApp() }

    @Test
    fun library() = render("phone-library.png") { Chrome { LibraryTab(onOpenShelf = {}) } }

    @Test
    fun settings() = render("phone-settings.png") { Chrome { SettingsTab(onOpen = {}) } }

    @Test
    fun appearance() = render("phone-settings-appearance.png") { Chrome { SettingsPageScreen(MobileSettingsPage.Appearance, onBack = {}) } }

    @Test
    fun details() = render("phone-details.png") { Chrome { DetailsScreen("1", onBack = {}, onOpenReader = { _, _ -> }) } }

    @Test
    fun reader() {
        val pdf = SamplePdfRepository()
        val vm = ReaderViewModel("3", "The hippocampus as a spatial map", SampleReaderRepository(fakePdfs = true), pdf)
        val state = runBlocking { withTimeout(10_000) { vm.state.first { !it.isLoading && !it.pdf.isLoading } } }
        val path = state.pdf.path!!
        render("phone-reader.png") {
            Theme {
                MobileReaderView(
                    state = state,
                    onEvent = {},
                    loadPage = { index, width -> pdf.render(path, index, width).toImageBitmap() },
                    loadFigure = { null },
                    renderedNear = { _, _ -> null },
                    textLayer = vm,
                    onSelectionChange = {},
                    onBack = {},
                )
            }
        }
    }

    @Composable
    private fun Theme(content: @Composable () -> Unit) = QuireTheme(appTheme = AppTheme.Obsidian, content = content)

    /** What MobileApp gives its screens: the theme, the bars' haze, the bottom bar's space. */
    @Composable
    private fun Chrome(content: @Composable () -> Unit) = Theme {
        CompositionLocalProvider(LocalMobileChrome provides MobileChrome(rememberHazeState(), bottomBar = 80.dp)) { content() }
    }

    private fun render(name: String, content: @Composable () -> Unit) {
        val png = ImageComposeScene(1081, 2402, Density(2.625f), content = content).use { scene ->
            // The first frames load resources and pages; later ones draw them (sheets animate in).
            scene.render()
            Thread.sleep(400)
            scene.render(600_000_000L)
            Thread.sleep(400)
            scene.render(1_500_000_000L).encodeToData(EncodedImageFormat.PNG)!!.bytes
        }
        val file = File("build/screenshots/$name").apply { parentFile.mkdirs(); writeBytes(png) }
        assertTrue(file.length() > 0)
        assets()?.let { File(it, name).writeBytes(png) }
    }

    private fun assets(): File? {
        var dir: File? = File(".").canonicalFile
        while (dir != null) {
            val candidate = File(dir, "assets")
            if (candidate.isDirectory) return File(candidate, "gui").apply { mkdirs() }
            dir = dir.parentFile
        }
        return null
    }
}
