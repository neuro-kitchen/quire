package dev.neurokitchen.quire

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyShortcut
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.MenuBar
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import dev.neurokitchen.quire.data.engine.engineDataModule
import dev.neurokitchen.quire.di.initKoin
import dev.neurokitchen.quire.domain.repository.LibraryRepository
import dev.neurokitchen.quire.presentation.desktop.DesktopApp
import dev.neurokitchen.quire.presentation.desktop.DesktopSettings
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.menu_edit
import dev.neurokitchen.quire.resources.menu_file
import dev.neurokitchen.quire.resources.menu_quit
import dev.neurokitchen.quire.resources.menu_reload
import dev.neurokitchen.quire.resources.menu_settings
import dev.neurokitchen.quire.resources.menu_view
import dev.neurokitchen.quire.resources.settings_title
import kotlinx.coroutines.launch
import org.jetbrains.compose.resources.stringResource
import org.koin.core.context.GlobalContext

fun main() {
    initKoin(data = engineDataModule)
    application {
        var settingsOpen by remember { mutableStateOf(false) }
        val scope = rememberCoroutineScope()
        val library = remember { GlobalContext.get().get<LibraryRepository>() }
        Window(
            onCloseRequest = ::exitApplication,
            title = "quire",
            state = rememberWindowState(size = DpSize(1400.dp, 860.dp)),
        ) {
            MenuBar {
                Menu(stringResource(Res.string.menu_file)) {
                    Item(stringResource(Res.string.menu_quit), shortcut = KeyShortcut(Key.Q, ctrl = true), onClick = ::exitApplication)
                }
                Menu(stringResource(Res.string.menu_edit)) {
                    Item(stringResource(Res.string.menu_settings), shortcut = KeyShortcut(Key.Comma, ctrl = true)) {
                        settingsOpen = true
                    }
                }
                Menu(stringResource(Res.string.menu_view)) {
                    // Changes made outside the app (`quire` in a terminal, another device's sync)
                    // are not announced to this engine: reopen the library to see them.
                    Item(stringResource(Res.string.menu_reload), shortcut = KeyShortcut(Key.R, ctrl = true)) {
                        scope.launch { library.reopen() }
                    }
                }
            }
            DesktopApp()
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
