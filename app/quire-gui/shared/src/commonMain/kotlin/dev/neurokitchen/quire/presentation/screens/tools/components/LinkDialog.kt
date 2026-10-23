package dev.neurokitchen.quire.presentation.screens.tools.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.screens.tools.LinkingState
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tools_link_allow
import dev.neurokitchen.quire.resources.tools_link_ask_body
import dev.neurokitchen.quire.resources.tools_link_ask_title
import dev.neurokitchen.quire.resources.tools_link_copy
import dev.neurokitchen.quire.resources.tools_link_join_body
import dev.neurokitchen.quire.resources.tools_link_join_connecting
import dev.neurokitchen.quire.resources.tools_link_join_title
import dev.neurokitchen.quire.resources.tools_link_qr
import dev.neurokitchen.quire.resources.tools_link_refuse
import dev.neurokitchen.quire.resources.tools_link_show_body
import dev.neurokitchen.quire.resources.tools_link_show_title
import io.github.alexzhirkevich.qrose.rememberQrCodePainter
import org.jetbrains.compose.resources.stringResource

/**
 * Linking a device (`.tasks/T14-security.md`), on desktop and phone: this device's code as a QR
 * code; then who asks, with the check words, to allow or refuse; or, joining, the words to compare
 * while the other device decides.
 */
@Composable
internal fun LinkDialog(linking: LinkingState, onEvent: (ToolsEvent) -> Unit) {
    val cancel = { onEvent(ToolsEvent.OnCancelLink) }
    when (linking) {
        is LinkingState.Showing -> {
            val clipboard = LocalClipboardManager.current
            DialogShell(
                title = stringResource(Res.string.tools_link_show_title),
                subtitle = stringResource(Res.string.tools_link_show_body),
                width = 420.dp,
                onDismiss = cancel,
                actions = {
                    TextButton(onClick = { clipboard.setText(AnnotatedString(linking.uri)) }) { Text(stringResource(Res.string.tools_link_copy)) }
                    DialogCancel(cancel)
                },
            ) {
                // Dark modules on white, whatever the theme: scanners need the contrast.
                Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                    Box(Modifier.clip(RoundedCornerShape(Dimens.RadiusBase)).background(Color.White).padding(Dimens.SpaceSm)) {
                        Image(
                            painter = rememberQrCodePainter(linking.uri),
                            contentDescription = stringResource(Res.string.tools_link_qr),
                            modifier = Modifier.size(240.dp),
                        )
                    }
                }
            }
        }
        is LinkingState.Asking -> DialogShell(
            title = stringResource(Res.string.tools_link_ask_title, linking.name),
            subtitle = stringResource(Res.string.tools_link_ask_body),
            width = 420.dp,
            onDismiss = { onEvent(ToolsEvent.OnRefuseLink) },
            actions = {
                DialogCancel({ onEvent(ToolsEvent.OnRefuseLink) }, stringResource(Res.string.tools_link_refuse))
                DialogConfirm(stringResource(Res.string.tools_link_allow), { onEvent(ToolsEvent.OnAllowLink) })
            },
        ) { CheckWords(linking.words) }
        is LinkingState.Joining -> DialogShell(
            title = stringResource(Res.string.tools_link_join_title),
            subtitle = stringResource(if (linking.words.isEmpty()) Res.string.tools_link_join_connecting else Res.string.tools_link_join_body),
            width = 420.dp,
            onDismiss = cancel,
            actions = { DialogCancel(cancel) },
        ) {
            if (linking.words.isEmpty()) {
                Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) { CircularProgressIndicator(Modifier.size(32.dp)) }
            } else {
                CheckWords(linking.words)
            }
        }
    }
}

/** The check words, large, to compare between the two screens. */
@Composable
private fun CheckWords(words: List<String>) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm, Alignment.CenterHorizontally)) {
        words.forEach { word ->
            Column(
                Modifier.clip(RoundedCornerShape(Dimens.RadiusBase)).background(QuireTheme.colors.surfaceContainer).padding(horizontal = Dimens.SpaceSm, vertical = Dimens.SpaceXs),
            ) {
                Text(word, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = QuireTheme.colors.fillPrimary)
            }
        }
    }
}
