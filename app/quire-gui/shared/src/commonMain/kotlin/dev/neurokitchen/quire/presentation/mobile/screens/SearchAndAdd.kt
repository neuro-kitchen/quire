package dev.neurokitchen.quire.presentation.mobile.screens

import dev.neurokitchen.quire.presentation.screens.library.AddTab
import org.jetbrains.compose.resources.stringResource
import dev.neurokitchen.quire.resources.add_confirm_short
import dev.neurokitchen.quire.resources.add_subtitle_phone
import dev.neurokitchen.quire.resources.add_title
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.presentation.screens.library.components.AddReferenceContent
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.chrisbanes.haze.hazeSource
import androidx.compose.ui.unit.dp
import androidx.compose.material3.TextButton
import androidx.compose.material3.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.background
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

/**
 * Adding references, full screen (Material's full-screen dialog): close on the left, Add on the
 * right of the top bar; every way in as scrollable tabs. Closes itself when the add is done.
 */
@Composable
fun AddScreen(
    projectPath: String?,
    onDone: () -> Unit,
    /** What was shared with Quire: identifiers (one per line) or PDF files. */
    identifiers: String = "",
    pdfs: List<String> = emptyList(),
    /** The tab to open on (the Add menu's choice). */
    way: AddTab? = null,
    viewModel: LibraryViewModel = koinViewModel(),
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LaunchedEffect(Unit) {
        viewModel.onEvent(LibraryEvent.OnShowAddDialog(true))
        projectPath?.let { viewModel.onEvent(LibraryEvent.OnAddProjectSelect(it)) }
        way?.let { viewModel.onEvent(LibraryEvent.OnAddTabSelect(it)) }
        if (pdfs.isNotEmpty()) {
            viewModel.onEvent(LibraryEvent.OnAddTabSelect(AddTab.Pdf))
            viewModel.onEvent(LibraryEvent.OnAddPdfsPicked(pdfs))
        } else if (identifiers.isNotBlank()) {
            viewModel.onEvent(LibraryEvent.OnIdentifierChange(identifiers))
        }
    }
    val dialog = state.addDialog
    // The add went through: its state, once shown, is cleared; back to where Add was opened.
    // (Not before it was shown: the ViewModel's state arrives a moment after the request.)
    var shown by remember { mutableStateOf(false) }
    LaunchedEffect(dialog == null) {
        if (dialog != null) shown = true else if (shown) onDone()
    }
    val chrome = LocalMobileChrome.current
    Box(Modifier.fillMaxSize().background(QuireTheme.colors.background)) {
        if (dialog != null) {
            Column(
                Modifier
                    .fillMaxSize()
                    .hazeSource(chrome.haze)
                    .imePadding()
                    .verticalScroll(rememberScrollState())
                    .padding(mobileContentPadding())
                    .padding(horizontal = Dimens.SpaceMd),
            ) {
                Text(stringResource(Res.string.add_subtitle_phone), style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary)
                Spacer(Modifier.height(Dimens.SpaceMd))
                AddReferenceContent(dialog, viewModel::onEvent, state.projectChoices, scrollableTabs = true)
            }
        }
        MobileTopBar(stringResource(Res.string.add_title), onBack = onDone, haze = chrome.haze, actions = {
            if (dialog?.isWorking == true) CircularProgressIndicator(Modifier.size(20.dp), strokeWidth = 2.dp)
            TextButton(onClick = { viewModel.onEvent(LibraryEvent.OnAddConfirm) }, enabled = dialog?.canAdd == true && !dialog.isWorking) {
                Text(stringResource(Res.string.add_confirm_short))
            }
        })
    }
}
