package dev.neurokitchen.quire.di

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.neurokitchen.quire.data.engine.EngineLibraryRepository
import dev.neurokitchen.quire.data.engine.EngineSession
import dev.neurokitchen.quire.data.engine.engineDataModule
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.ffi.openLibraryAt
import dev.neurokitchen.quire.presentation.desktop.DesktopViewModel
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.reader.ReaderViewModel
import dev.neurokitchen.quire.presentation.screens.settings.SettingsViewModel
import dev.neurokitchen.quire.presentation.screens.tools.ToolsViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.job
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.koin.core.context.stopKoin
import org.koin.core.parameter.parametersOf
import org.koin.dsl.module
import java.nio.file.Files
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertIs
import kotlin.test.assertSame

/**
 * The app's own graph on the engine: every screen's ViewModel resolves, and the library is the
 * engine's. (A switch to the sample data in the app once went unnoticed: tests built the engine
 * repositories directly.) The session points at a temporary library, never the user's.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class EngineWiringTest {
    @BeforeTest
    fun setUp() = Dispatchers.setMain(Dispatchers.Default)

    @AfterTest
    fun tearDown() {
        stopKoin()
        Dispatchers.resetMain()
    }

    @Test
    fun everyScreenResolvesOnTheEngine() = runBlocking {
        val dir = Files.createTempDirectory("quire-test")
        val session = EngineSession(open = { openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString()) })
        val koin = initKoin(data = engineDataModule).koin
        // Loaded after the app's modules (and before anything is resolved), so it replaces theirs.
        koin.loadModules(listOf(module { single { session } }), allowOverride = true)
        assertSame(session, koin.get<EngineSession>(), "the temporary library, never the user's")
        assertIs<EngineLibraryRepository>(koin.get<LibraryRepository>())
        val viewModels: List<ViewModel> = listOf(
            koin.get<LibraryViewModel>(),
            koin.get<ToolsViewModel>(),
            koin.get<SettingsViewModel>(),
            koin.get<DesktopViewModel>(),
            koin.get<ReaderViewModel> { parametersOf("missing", "A paper") },
        )
        viewModels.forEach { it.viewModelScope.coroutineContext.job.cancelAndJoin() }
    }
}
