package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.screens.notifications.NotificationsEvent
import dev.neurokitchen.quire.presentation.screens.notifications.NotificationsViewModel
import dev.neurokitchen.quire.presentation.screens.notifications.components.NotificationsList
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.notifications
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** Settings › Notifications on the phone: the list (read once shown). */
@Composable
fun NotificationsPageScreen(onBack: () -> Unit, viewModel: NotificationsViewModel = koinViewModel()) {
    DisposableEffect(Unit) {
        viewModel.onEvent(NotificationsEvent.OnOpen)
        onDispose { viewModel.onEvent(NotificationsEvent.OnClose) }
    }
    val state by viewModel.state.collectAsStateWithLifecycle()
    val chrome = LocalMobileChrome.current
    Box(Modifier.fillMaxSize()) {
        LazyColumn(Modifier.fillMaxSize().hazeSource(chrome.haze), contentPadding = mobileContentPadding()) {
            item { NotificationsList(state.items.orEmpty(), Modifier.padding(horizontal = Dimens.SpaceMd)) }
        }
        MobileTopBar(stringResource(Res.string.notifications), onBack, chrome.haze)
    }
}
