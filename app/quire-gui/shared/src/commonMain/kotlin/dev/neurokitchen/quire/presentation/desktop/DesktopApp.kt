package dev.neurokitchen.quire.presentation.desktop

import dev.neurokitchen.quire.presentation.components.commandpalette.CommandPaletteDialog
import dev.neurokitchen.quire.presentation.components.graph.CitationGraphDialog
import dev.neurokitchen.quire.presentation.screens.tools.ToolsRoute
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tools_title
import org.jetbrains.compose.resources.stringResource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
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
        Surface(
            modifier = Modifier
                .fillMaxSize()
                .onPreviewKeyEvent { event ->
                    if (event.type == KeyEventType.KeyDown && (event.isCtrlPressed || event.isMetaPressed)) {
                        when (event.key) {
                            Key.K -> {
                                viewModel.onEvent(DesktopEvent.OnToggleCommandPalette(!state.isCommandPaletteOpen))
                                true
                            }
                            Key.G -> {
                                viewModel.onEvent(DesktopEvent.OnToggleCitationGraph(!state.isCitationGraphOpen))
                                true
                            }
                            else -> false
                        }
                    } else false
                },
        ) {
            Box(Modifier.fillMaxSize()) {
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
                        onCommandPaletteClick = { viewModel.onEvent(DesktopEvent.OnToggleCommandPalette(true)) },
                        onCitationGraphClick = { viewModel.onEvent(DesktopEvent.OnToggleCitationGraph(true)) },
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

                // Global Command Palette Overlay
                CommandPaletteDialog(
                    isOpen = state.isCommandPaletteOpen,
                    onDismissRequest = { viewModel.onEvent(DesktopEvent.OnToggleCommandPalette(false)) },
                    papers = state.papers,
                    allReferences = state.allReferences,
                    currentTheme = state.appTheme,
                    onOpenPaper = { id, title -> viewModel.onEvent(DesktopEvent.OnOpenPaper(id, title)) },
                    onSelectTab = { id -> viewModel.onEvent(DesktopEvent.OnSelectTab(id)) },
                    onSetTheme = { theme -> viewModel.onEvent(DesktopEvent.OnSetAppTheme(theme)) },
                    onOpenSettings = onOpenSettings,
                    onReload = onReload,
                    onOpenTools = { viewModel.onEvent(DesktopEvent.OnOpenPaper(ToolsTabId, toolsTitle)) },
                    onOpenCitationGraph = { viewModel.onEvent(DesktopEvent.OnToggleCitationGraph(true)) },
                )

                // Local Citation Network Graph
                CitationGraphDialog(
                    isOpen = state.isCitationGraphOpen,
                    onDismissRequest = { viewModel.onEvent(DesktopEvent.OnToggleCitationGraph(false)) },
                    references = state.allReferences,
                    onOpenPaper = { id, title -> viewModel.onEvent(DesktopEvent.OnOpenPaper(id, title)) },
                )
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
