package dev.neurokitchen.quire.presentation.screens.tools.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.CompactTextField
import dev.neurokitchen.quire.presentation.platform.deviceName
import dev.neurokitchen.quire.presentation.platform.rememberQrScanner
import dev.neurokitchen.quire.presentation.screens.tools.ToolsConfirm
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tools_device_id
import dev.neurokitchen.quire.resources.tools_hide_key
import dev.neurokitchen.quire.resources.tools_hub
import dev.neurokitchen.quire.resources.tools_hub_devices
import dev.neurokitchen.quire.resources.tools_hub_hint
import dev.neurokitchen.quire.resources.tools_hub_id
import dev.neurokitchen.quire.resources.tools_hub_join
import dev.neurokitchen.quire.resources.tools_hub_joined
import dev.neurokitchen.quire.resources.tools_hub_link
import dev.neurokitchen.quire.resources.tools_hub_link_hint
import dev.neurokitchen.quire.resources.tools_hub_load_devices
import dev.neurokitchen.quire.resources.tools_hub_over_quota
import dev.neurokitchen.quire.resources.tools_hub_registered
import dev.neurokitchen.quire.resources.tools_hub_report
import dev.neurokitchen.quire.resources.tools_hub_revoked_mark
import dev.neurokitchen.quire.resources.tools_hub_this
import dev.neurokitchen.quire.resources.tools_recovery_kit
import dev.neurokitchen.quire.resources.tools_recovery_kit_hint
import dev.neurokitchen.quire.resources.tools_relay
import dev.neurokitchen.quire.resources.tools_relay_hint
import dev.neurokitchen.quire.resources.tools_revoke
import dev.neurokitchen.quire.resources.tools_save
import dev.neurokitchen.quire.resources.tools_setup_code
import dev.neurokitchen.quire.resources.tools_show_key
import dev.neurokitchen.quire.resources.tools_sync_hub
import dev.neurokitchen.quire.resources.tools_join
import dev.neurokitchen.quire.resources.tools_join_code
import dev.neurokitchen.quire.resources.tools_link_device
import dev.neurokitchen.quire.resources.tools_link_hint
import dev.neurokitchen.quire.resources.tools_scan_code
import dev.neurokitchen.quire.resources.tools_this_device
import dev.neurokitchen.quire.resources.tools_user_id
import org.jetbrains.compose.resources.stringResource

/**
 * This device's IDs, and linking: show a code for a new device, or join another device's code
 * (scanned where there is a camera, else pasted). No key is ever shown (`.tasks/T14-security.md`).
 */
@Composable
internal fun ThisDeviceGroup(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    val sync = state.sync
    val name = deviceName()
    val scanner = rememberQrScanner { code -> onEvent(ToolsEvent.OnJoinLink(code, name)) }
    Text(stringResource(Res.string.tools_this_device), style = MaterialTheme.typography.titleSmall, color = QuireTheme.colors.fillPrimary)
    Group {
        sync.deviceId?.let { CopyRow(Res.string.tools_device_id, it) }
        sync.userId?.let { CopyRow(Res.string.tools_user_id, it) }
        Spacer(Modifier.height(Dimens.SpaceXs))
        Hint(stringResource(Res.string.tools_link_hint))
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            SmallAction(Res.string.tools_link_device, enabled = sync.linking == null && !state.isWorking) { onEvent(ToolsEvent.OnLinkDevice) }
            if (scanner.canScan) {
                SmallAction(Res.string.tools_scan_code, enabled = sync.linking == null && !state.isWorking) { scanner.scan() }
            }
        }
        var code by remember { mutableStateOf("") }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            CompactTextField(value = code, onValueChange = { code = it }, placeholder = stringResource(Res.string.tools_join_code), modifier = Modifier.weight(1f))
            SmallAction(Res.string.tools_join, enabled = code.isNotBlank() && sync.linking == null && !state.isWorking) {
                onEvent(ToolsEvent.OnJoinLink(code, name))
                code = ""
            }
        }
    }
}

/**
 * The sync hub: its ID and our own relay, syncing (with the setup code the first time), what the
 * last sync did, this person's devices there, and the recovery kit. Fields stack, so it fits a phone.
 */
@Composable
internal fun HubGroup(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    val sync = state.sync
    Text(stringResource(Res.string.tools_hub), style = MaterialTheme.typography.titleSmall, color = QuireTheme.colors.fillPrimary)
    Hint(stringResource(Res.string.tools_hub_hint))
    Spacer(Modifier.height(Dimens.SpaceXs))
    Group {
        // The usual way: the link the hub prints (or its QR code); the fields below are the same by hand.
        var link by remember { mutableStateOf("") }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            CompactTextField(value = link, onValueChange = { link = it }, placeholder = stringResource(Res.string.tools_hub_link), modifier = Modifier.weight(1f))
            SmallAction(Res.string.tools_hub_join, enabled = link.isNotBlank() && !state.isWorking) {
                onEvent(ToolsEvent.OnUseHubLink(link))
                link = ""
            }
        }
        Hint(stringResource(Res.string.tools_hub_link_hint))
        Spacer(Modifier.height(Dimens.SpaceXs))
        // Keyed on the saved values: the fields show them again after a save or a reload.
        var hubId by remember(sync.hubId) { mutableStateOf(sync.hubId.orEmpty()) }
        var relay by remember(sync.relay) { mutableStateOf(sync.relay.orEmpty()) }
        val changed = hubId.trim() != sync.hubId.orEmpty() || relay.trim() != sync.relay.orEmpty()
        CompactTextField(value = hubId, onValueChange = { hubId = it }, placeholder = stringResource(Res.string.tools_hub_id), modifier = Modifier.fillMaxWidth())
        CompactTextField(value = relay, onValueChange = { relay = it }, placeholder = stringResource(Res.string.tools_relay), modifier = Modifier.fillMaxWidth())
        Row(verticalAlignment = Alignment.CenterVertically) {
            Hint(stringResource(Res.string.tools_relay_hint), Modifier.weight(1f))
            SmallAction(Res.string.tools_save, enabled = changed && !state.isWorking) { onEvent(ToolsEvent.OnSaveHub(hubId, relay)) }
        }

        Spacer(Modifier.height(Dimens.SpaceXs))
        var code by remember { mutableStateOf("") }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            CompactTextField(value = code, onValueChange = { code = it }, placeholder = stringResource(Res.string.tools_setup_code), modifier = Modifier.weight(1f))
            SmallAction(Res.string.tools_sync_hub, enabled = sync.hubId != null && !changed && !state.isWorking) {
                onEvent(ToolsEvent.OnSyncHub(code))
                code = ""
            }
        }
        sync.hubReport?.let { r ->
            if (r.registered) Hint(stringResource(Res.string.tools_hub_registered))
            if (r.joined) Hint(stringResource(Res.string.tools_hub_joined))
            Hint(stringResource(Res.string.tools_hub_report, r.pushed, r.applied, r.filesUploaded, r.filesDownloaded))
            if (r.overQuota > 0) Hint(stringResource(Res.string.tools_hub_over_quota, r.overQuota))
        }
    }

    if (sync.hubId == null) return
    Spacer(Modifier.height(Dimens.SpaceXs))
    Group {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(stringResource(Res.string.tools_hub_devices), style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary, modifier = Modifier.weight(1f))
            SmallAction(Res.string.tools_hub_load_devices, enabled = !state.isWorking) { onEvent(ToolsEvent.OnLoadHubDevices) }
        }
        sync.hubDevices?.forEach { device ->
            val isThis = device.device == sync.deviceId
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text(device.device, style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace), color = QuireTheme.colors.fillPrimary, maxLines = 1)
                    val marks = listOfNotNull(
                        stringResource(Res.string.tools_hub_this).takeIf { isThis },
                        stringResource(Res.string.tools_hub_revoked_mark).takeIf { device.revoked },
                    )
                    if (marks.isNotEmpty()) Hint(marks.joinToString(" · "))
                }
                if (!isThis && !device.revoked) {
                    SmallAction(Res.string.tools_revoke, enabled = !state.isWorking, color = QuireTheme.colors.accentRed) {
                        onEvent(ToolsEvent.OnConfirm(ToolsConfirm.RevokeHubDevice(device.device)))
                    }
                }
            }
        }
        Spacer(Modifier.height(Dimens.SpaceXs))
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                Text(stringResource(Res.string.tools_recovery_kit), style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary)
                Hint(stringResource(Res.string.tools_recovery_kit_hint))
            }
            if (sync.recoveryKit == null) {
                SmallAction(Res.string.tools_show_key, enabled = !state.isWorking) { onEvent(ToolsEvent.OnRevealRecoveryKit) }
            } else {
                SmallAction(Res.string.tools_hide_key) { onEvent(ToolsEvent.OnHideRecoveryKit) }
            }
        }
        sync.recoveryKit?.let { CopyRow(Res.string.tools_recovery_kit, it) }
    }
}
