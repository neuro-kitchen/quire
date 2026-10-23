package dev.neurokitchen.quire.presentation.screens.notifications.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.ToolbarIconButton
import dev.neurokitchen.quire.presentation.screens.notifications.NotificationsEvent
import dev.neurokitchen.quire.presentation.screens.notifications.NotificationsViewModel
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_notifications
import dev.neurokitchen.quire.resources.notifications
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/** The desktop's bell: the unread count on it; a click shows the notifications (and reads them). */
@Composable
fun NotificationsBell(viewModel: NotificationsViewModel = koinViewModel()) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val colors = QuireTheme.colors
    Box {
        ToolbarIconButton(Res.drawable.ic_notifications, stringResource(Res.string.notifications), onClick = { viewModel.onEvent(NotificationsEvent.OnOpen) })
        if (state.unread > 0) {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier.align(Alignment.TopEnd).offset(x = 2.dp, y = (-2).dp).background(colors.accentRed, CircleShape).padding(horizontal = 4.dp),
            ) {
                Text(if (state.unread > 99) "99+" else state.unread.toString(), style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp), color = colors.onAccent)
            }
        }
        DropdownMenu(
            expanded = state.isOpen,
            onDismissRequest = { viewModel.onEvent(NotificationsEvent.OnClose) },
            modifier = Modifier.width(380.dp).background(colors.sidepane).border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)),
        ) {
            Column(Modifier.padding(horizontal = Dimens.SpaceSm, vertical = Dimens.SpaceXs)) {
                Text(stringResource(Res.string.notifications), style = MaterialTheme.typography.titleSmall, color = colors.fillPrimary)
                NotificationsList(
                    state.items.orEmpty(),
                    Modifier.padding(top = Dimens.SpaceXs).heightIn(max = 480.dp).verticalScroll(rememberScrollState()),
                )
            }
        }
    }
}
