package dev.neurokitchen.quire.presentation.mobile.screens

import dev.neurokitchen.quire.resources.add_screen_manual
import dev.neurokitchen.quire.resources.add_screen_paste
import dev.neurokitchen.quire.resources.add_screen_bib
import dev.neurokitchen.quire.resources.add_screen_pdf
import dev.neurokitchen.quire.resources.add_screen_identifier
import org.jetbrains.compose.resources.StringResource
import dev.neurokitchen.quire.presentation.screens.library.AddTab
import org.jetbrains.compose.resources.stringResource
import dev.neurokitchen.quire.resources.add_confirm_short
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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.fillMaxSize
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
 * right of the top bar. One way in per screen (the Add menu's choice; shared PDFs: PDF files, else
 * by identifier), titled for it. Closes itself when the add is done.
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
    val shownWay = way ?: if (pdfs.isNotEmpty()) AddTab.Pdf else AddTab.Identifier
    LaunchedEffect(Unit) {
        viewModel.onEvent(LibraryEvent.OnShowAddDialog(true))
        projectPath?.let { viewModel.onEvent(LibraryEvent.OnAddProjectSelect(it)) }
        viewModel.onEvent(LibraryEvent.OnAddTabSelect(shownWay))
        if (pdfs.isNotEmpty()) {
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
                AddReferenceContent(dialog, viewModel::onEvent, state.projectChoices, showTabs = false)
            }
        }
        MobileTopBar(stringResource(shownWay.screenTitle), onBack = onDone, haze = chrome.haze, actions = {
            if (dialog?.isWorking == true) CircularProgressIndicator(Modifier.size(20.dp), strokeWidth = 2.dp)
            TextButton(onClick = { viewModel.onEvent(LibraryEvent.OnAddConfirm) }, enabled = dialog?.canAdd == true && !dialog.isWorking) {
                Text(stringResource(Res.string.add_confirm_short))
            }
        })
    }
}

/** An Add screen's title: what it adds by. */
private val AddTab.screenTitle: StringResource
    get() = when (this) {
        AddTab.Identifier -> Res.string.add_screen_identifier
        AddTab.Pdf -> Res.string.add_screen_pdf
        AddTab.File -> Res.string.add_screen_bib
        AddTab.Bibtex -> Res.string.add_screen_paste
        AddTab.Manual -> Res.string.add_screen_manual
    }
