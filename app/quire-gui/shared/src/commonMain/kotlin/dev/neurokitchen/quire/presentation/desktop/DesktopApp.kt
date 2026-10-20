package dev.neurokitchen.quire.presentation.desktop

import dev.neurokitchen.quire.presentation.screens.tools.ToolsRoute
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tools_title
import org.jetbrains.compose.resources.stringResource
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.desktop.components.TabBar
import dev.neurokitchen.quire.presentation.screens.library.LibraryRoute
import dev.neurokitchen.quire.presentation.screens.reader.ReaderRoute
import dev.neurokitchen.quire.presentation.screens.settings.SettingsRoute
import org.koin.compose.viewmodel.koinViewModel

/** The desktop main window's content: tab bar above the library or an open paper. */
@Composable
fun DesktopApp(
    viewModel: DesktopViewModel = koinViewModel(),
    onOpenSettings: () -> Unit = {},
    onReload: () -> Unit = {},
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val toolsTitle = stringResource(Res.string.tools_title)
    QuireTheme(appTheme = state.appTheme, desktop = true) {
        Surface {
            Column(Modifier.fillMaxSize()) {
                TabBar(
                    papers = state.papers,
                    selected = state.selected,
                    currentTheme = state.appTheme,
                    onSelect = { viewModel.onEvent(DesktopEvent.OnSelectTab(it)) },
                    onClose = { viewModel.onEvent(DesktopEvent.OnCloseTab(it)) },
                    onSyncClick = onReload,
                    onSettingsClick = onOpenSettings,
                    onToolsClick = { viewModel.onEvent(DesktopEvent.OnOpenPaper(ToolsTabId, toolsTitle)) },
                    onSetTheme = { viewModel.onEvent(DesktopEvent.OnSetAppTheme(it)) },
                )
                HorizontalDivider(color = QuireTheme.colors.paneDivider)
                val paper = state.papers.firstOrNull { it.id == state.selected }
                if (paper == null) {
                    LibraryRoute(onOpenReference = { id, title -> viewModel.onEvent(DesktopEvent.OnOpenPaper(id, title)) })
                } else if (paper.id == ToolsTabId) {
                    ToolsRoute()
                } else {
                    ReaderRoute(paper.id, paper.title)
                }
            }
        }
    }
}

/** The desktop Settings window's content. */
@Composable
fun DesktopSettings(
    viewModel: DesktopViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    QuireTheme(appTheme = state.appTheme, desktop = true) {
        Surface { SettingsRoute() }
    }
}
