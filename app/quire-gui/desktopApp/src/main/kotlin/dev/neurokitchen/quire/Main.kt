package dev.neurokitchen.quire

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import dev.neurokitchen.quire.di.initKoin
import dev.neurokitchen.quire.data.engine.engineDataModule
import dev.neurokitchen.quire.di.sampleDataModule
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.presentation.desktop.DesktopApp
import dev.neurokitchen.quire.presentation.desktop.DesktopSettings
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.settings_title
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.stringResource
import org.koin.core.context.GlobalContext

fun main() {
    // The real library through the engine; QUIRE_SAMPLE_DATA=1 shows the fixed sample library
    // instead (designing screens without a library).
    initKoin(data = if (System.getenv("QUIRE_SAMPLE_DATA") == "1") sampleDataModule else engineDataModule)
    application {
        var settingsOpen by remember { mutableStateOf(false) }
        val scope = rememberCoroutineScope()
        val library = remember { GlobalContext.get().get<LibraryRepository>() }
        Window(
            onCloseRequest = ::exitApplication,
            title = "quire",
            state = rememberWindowState(size = DpSize(1400.dp, 860.dp)),
            onKeyEvent = { keyEvent ->
                if (keyEvent.type == KeyEventType.KeyDown && keyEvent.isCtrlPressed) {
                    when (keyEvent.key) {
                        Key.Q -> {
                            exitApplication()
                            true
                        }
                        Key.Comma -> {
                            settingsOpen = true
                            true
                        }
                        Key.R -> {
                            scope.launch { library.reopen() }
                            true
                        }
                        else -> false
                    }
                } else {
                    false
                }
            },
        ) {
            DesktopApp(
                onOpenSettings = { settingsOpen = true },
                onReload = { scope.launch { library.reopen() } },
            )
        }
        if (settingsOpen) {
            Window(
                onCloseRequest = { settingsOpen = false },
                title = stringResource(Res.string.settings_title),
                state = rememberWindowState(size = DpSize(640.dp, 480.dp)),
            ) {
                DesktopSettings()
            }
        }
    }
}
