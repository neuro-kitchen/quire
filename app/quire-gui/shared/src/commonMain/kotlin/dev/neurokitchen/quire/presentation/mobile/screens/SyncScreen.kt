package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.presentation.components.notice.NoticeBar
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.platform.RequestLocalNetworkAccess
import dev.neurokitchen.quire.presentation.screens.tools.Confirm
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsSection
import dev.neurokitchen.quire.presentation.screens.tools.ToolsViewModel
import dev.neurokitchen.quire.presentation.screens.tools.components.HubGroup
import dev.neurokitchen.quire.presentation.screens.tools.components.LinkDialog
import dev.neurokitchen.quire.presentation.screens.tools.components.ThisDeviceGroup
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.mobile_settings_sync
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** Settings › Sync on the phone: this device's identity and the sync hub, as in the desktop's Tools › Sync. */
@Composable
fun SyncPageScreen(onBack: () -> Unit, viewModel: ToolsViewModel = koinViewModel()) {
    LaunchedEffect(Unit) { viewModel.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Sync)) }
    // The hub and relay may be at home: Android 17 needs the person's leave to reach them.
    RequestLocalNetworkAccess()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val onEvent = viewModel::onEvent
    val chrome = LocalMobileChrome.current
    Box(Modifier.fillMaxSize()) {
        LazyColumn(
            Modifier.fillMaxSize().hazeSource(chrome.haze),
            contentPadding = mobileContentPadding(),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
        ) {
            item { Column(Modifier.padding(horizontal = Dimens.SpaceMd)) { ThisDeviceGroup(state, onEvent) } }
            item { Column(Modifier.padding(horizontal = Dimens.SpaceMd)) { HubGroup(state, onEvent) } }
        }
        MobileTopBar(stringResource(Res.string.mobile_settings_sync), onBack, chrome.haze)
        state.notice?.let { notice ->
            NoticeBar(
                notice = notice,
                onDismiss = { onEvent(ToolsEvent.OnDismissNotice) },
                modifier = Modifier.align(Alignment.BottomCenter).padding(Dimens.SpaceMd),
            )
        }
    }
    state.confirm?.let { Confirm(it, onEvent) }
    state.sync.linking?.let { LinkDialog(it, onEvent) }
}
