package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.screens.library.LibraryEvent
import dev.neurokitchen.quire.presentation.screens.library.LibraryViewModel
import dev.neurokitchen.quire.presentation.screens.library.components.AddReferenceDialog
import dev.neurokitchen.quire.presentation.screens.search.SearchScreen
import org.koin.compose.viewmodel.koinViewModel

/** The Search tab: its own query (it never filters Home), results open their details. */
@Composable
fun SearchTab(
    onOpenDetails: (String) -> Unit,
    onOpenReader: (String, String) -> Unit,
    viewModel: LibraryViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    SearchScreen(
        query = state.query,
        onQueryChange = { viewModel.onEvent(LibraryEvent.OnQueryChange(it)) },
        references = state.references,
        onReferenceClick = onOpenDetails,
        onOpenReference = onOpenReader,
        modifier = Modifier.windowInsetsPadding(WindowInsets.statusBars),
        bottomPadding = LocalMobileChrome.current.bottomBar,
    )
}

/** Adding references; closes itself when the add is done or cancelled. */
@Composable
fun AddScreen(
    projectPath: String?,
    onDone: () -> Unit,
    viewModel: LibraryViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var opened by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) {
        viewModel.onEvent(LibraryEvent.OnShowAddDialog(true))
        projectPath?.let { viewModel.onEvent(LibraryEvent.OnAddProjectSelect(it)) }
        opened = true
    }
    val dialog = state.addDialog
    LaunchedEffect(dialog == null, opened) { if (opened && dialog == null) onDone() }
    if (dialog != null) AddReferenceDialog(dialog, viewModel::onEvent, projects = state.projectChoices)
}
