package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.chrisbanes.haze.hazeSource
import dev.neurokitchen.quire.domain.model.HubReport
import dev.neurokitchen.quire.presentation.components.notice.NoticeBar
import dev.neurokitchen.quire.presentation.components.time.ago
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsGroup
import dev.neurokitchen.quire.presentation.designsystem.components.settings.SettingsItem
import dev.neurokitchen.quire.presentation.mobile.components.LocalMobileChrome
import dev.neurokitchen.quire.presentation.mobile.components.MobileTopBar
import dev.neurokitchen.quire.presentation.mobile.components.mobileContentPadding
import dev.neurokitchen.quire.presentation.platform.RequestLocalNetworkAccess
import dev.neurokitchen.quire.presentation.platform.deviceName
import dev.neurokitchen.quire.presentation.platform.rememberQrScanner
import dev.neurokitchen.quire.presentation.screens.tools.Confirm
import dev.neurokitchen.quire.presentation.screens.tools.ToolsConfirm
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsSection
import dev.neurokitchen.quire.presentation.screens.tools.ToolsState
import dev.neurokitchen.quire.presentation.screens.tools.ToolsViewModel
import dev.neurokitchen.quire.presentation.screens.tools.components.LinkDialog
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_content_copy
import dev.neurokitchen.quire.resources.ic_info
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_person
import dev.neurokitchen.quire.resources.ic_sync
import dev.neurokitchen.quire.resources.ic_tune
import dev.neurokitchen.quire.resources.mobile_settings_sync
import dev.neurokitchen.quire.resources.mobile_sync_enter_code
import dev.neurokitchen.quire.resources.mobile_sync_enter_code_hint
import dev.neurokitchen.quire.resources.mobile_sync_manual_hub
import dev.neurokitchen.quire.resources.mobile_sync_manual_hub_hint
import dev.neurokitchen.quire.resources.mobile_sync_scan_hint
import dev.neurokitchen.quire.resources.name_cancel
import dev.neurokitchen.quire.resources.tools_copy
import dev.neurokitchen.quire.resources.tools_device_id
import dev.neurokitchen.quire.resources.tools_hide_ids
import dev.neurokitchen.quire.resources.tools_hide_key
import dev.neurokitchen.quire.resources.tools_hub
import dev.neurokitchen.quire.resources.tools_hub_change
import dev.neurokitchen.quire.resources.tools_hub_connected
import dev.neurokitchen.quire.resources.tools_hub_devices
import dev.neurokitchen.quire.resources.tools_hub_hint
import dev.neurokitchen.quire.resources.tools_hub_id
import dev.neurokitchen.quire.resources.tools_hub_join
import dev.neurokitchen.quire.resources.tools_hub_joined
import dev.neurokitchen.quire.resources.tools_hub_last_seen
import dev.neurokitchen.quire.resources.tools_hub_last_sync
import dev.neurokitchen.quire.resources.tools_hub_link
import dev.neurokitchen.quire.resources.tools_hub_link_hint
import dev.neurokitchen.quire.resources.tools_hub_load_devices
import dev.neurokitchen.quire.resources.tools_hub_n0_relay
import dev.neurokitchen.quire.resources.tools_hub_no_devices
import dev.neurokitchen.quire.resources.tools_hub_over_quota
import dev.neurokitchen.quire.resources.tools_hub_own_relay
import dev.neurokitchen.quire.resources.tools_hub_registered
import dev.neurokitchen.quire.resources.tools_hub_report
import dev.neurokitchen.quire.resources.tools_hub_revoked_mark
import dev.neurokitchen.quire.resources.tools_hub_this
import dev.neurokitchen.quire.resources.tools_hub_unnamed
import dev.neurokitchen.quire.resources.tools_ids_hint
import dev.neurokitchen.quire.resources.tools_join
import dev.neurokitchen.quire.resources.tools_join_code
import dev.neurokitchen.quire.resources.tools_link_device
import dev.neurokitchen.quire.resources.tools_link_hint
import dev.neurokitchen.quire.resources.tools_recovery_kit
import dev.neurokitchen.quire.resources.tools_recovery_kit_hint
import dev.neurokitchen.quire.resources.tools_relay
import dev.neurokitchen.quire.resources.tools_relay_hint
import dev.neurokitchen.quire.resources.tools_revoke
import dev.neurokitchen.quire.resources.tools_save
import dev.neurokitchen.quire.resources.tools_scan_code
import dev.neurokitchen.quire.resources.tools_setup_code
import dev.neurokitchen.quire.resources.tools_show_ids
import dev.neurokitchen.quire.resources.tools_show_key
import dev.neurokitchen.quire.resources.tools_sync_hub
import dev.neurokitchen.quire.resources.tools_this_device
import dev.neurokitchen.quire.resources.tools_user_id
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource
import org.koin.compose.viewmodel.koinViewModel

/**
 * Settings › Sync on the phone: Material 3 Expressive segmented groups for linking devices,
 * connecting to the sync hub, managing linked devices, and viewing the recovery kit.
 */
@Composable
fun SyncPageScreen(onBack: () -> Unit, viewModel: ToolsViewModel = koinViewModel()) {
    LaunchedEffect(Unit) { viewModel.onEvent(ToolsEvent.OnSectionSelect(ToolsSection.Sync)) }
    // The hub and relay may be at home: Android 17 needs the person's leave to reach them.
    RequestLocalNetworkAccess()
    val state by viewModel.state.collectAsStateWithLifecycle()
    val onEvent = viewModel::onEvent
    val chrome = LocalMobileChrome.current
    Box(Modifier.fillMaxSize().background(QuireTheme.colors.background)) {
        LazyColumn(
            Modifier.fillMaxSize().hazeSource(chrome.haze),
            contentPadding = mobileContentPadding(),
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
        ) {
            item { MobileThisDeviceGroup(state, onEvent) }
            item { MobileHubGroup(state, onEvent) }
            if (state.sync.hubId != null) {
                item { MobileHubDevicesGroup(state, onEvent) }
            }
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

/**
 * This device's linking actions and (hidden by default) identifiers in M3 Expressive rows.
 * No key is ever shown (`.tasks/T14-security.md`).
 */
@Composable
private fun MobileThisDeviceGroup(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    val sync = state.sync
    val name = deviceName()
    val scanner = rememberQrScanner { code -> onEvent(ToolsEvent.OnJoinLink(code, name)) }
    var showIds by remember { mutableStateOf(false) }
    var showJoinCodeDialog by remember { mutableStateOf(false) }
    val canAct = sync.linking == null && !state.isWorking

    val rows = buildList<@Composable (Shape) -> Unit> {
        add { shape ->
            SettingsItem(
                title = stringResource(Res.string.tools_link_device),
                shape = shape,
                supporting = stringResource(Res.string.tools_link_hint),
                icon = Res.drawable.ic_link,
                enabled = canAct,
                opensPage = true,
                onClick = { onEvent(ToolsEvent.OnLinkDevice) },
            )
        }
        if (scanner.canScan) {
            add { shape ->
                SettingsItem(
                    title = stringResource(Res.string.tools_scan_code),
                    shape = shape,
                    supporting = stringResource(Res.string.mobile_sync_scan_hint),
                    icon = Res.drawable.ic_sync,
                    enabled = canAct,
                    opensPage = true,
                    onClick = { scanner.scan() },
                )
            }
        }
        add { shape ->
            SettingsItem(
                title = stringResource(Res.string.mobile_sync_enter_code),
                shape = shape,
                supporting = stringResource(Res.string.mobile_sync_enter_code_hint),
                icon = Res.drawable.ic_content_copy,
                enabled = canAct,
                opensPage = true,
                onClick = { showJoinCodeDialog = true },
            )
        }
        add { shape ->
            SettingsItem(
                title = stringResource(if (showIds) Res.string.tools_hide_ids else Res.string.tools_show_ids),
                shape = shape,
                supporting = stringResource(Res.string.tools_ids_hint),
                icon = Res.drawable.ic_info,
                onClick = { showIds = !showIds },
            )
        }
        if (showIds) {
            sync.deviceId?.let { id ->
                add { shape -> MobileCopySegment(label = stringResource(Res.string.tools_device_id), value = id, shape = shape) }
            }
            sync.userId?.let { id ->
                add { shape -> MobileCopySegment(label = stringResource(Res.string.tools_user_id), value = id, shape = shape) }
            }
        }
    }

    SettingsGroup(
        label = stringResource(Res.string.tools_this_device),
        rows = rows,
    )

    if (showJoinCodeDialog) {
        JoinWithCodeDialog(
            working = state.isWorking,
            onDismiss = { showJoinCodeDialog = false },
            onJoin = { code ->
                showJoinCodeDialog = false
                onEvent(ToolsEvent.OnJoinLink(code, name))
            },
        )
    }
}

@Composable
private fun JoinWithCodeDialog(
    working: Boolean,
    onDismiss: () -> Unit,
    onJoin: (String) -> Unit,
) {
    var code by remember { mutableStateOf("") }
    DialogShell(
        title = stringResource(Res.string.tools_join),
        subtitle = stringResource(Res.string.tools_join_code),
        icon = Res.drawable.ic_link,
        width = 360.dp,
        onDismiss = onDismiss,
        actions = {
            DialogCancel(onDismiss)
            DialogConfirm(
                label = stringResource(Res.string.tools_join),
                onClick = { if (code.isNotBlank()) onJoin(code.trim()) },
                enabled = code.isNotBlank(),
                working = working,
            )
        },
    ) {
        OutlinedTextField(
            value = code,
            onValueChange = { code = it },
            placeholder = { Text("quire://device?…", color = QuireTheme.colors.fillTertiary) },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { if (code.isNotBlank()) onJoin(code.trim()) }),
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

/**
 * Sync hub setup & status: folds into a single connected card once joined, or shows the primary
 * `quire://hub?…` link input with an expandable manual ID & relay section.
 */
@Composable
private fun MobileHubGroup(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    val sync = state.sync
    val colors = QuireTheme.colors
    var changing by remember(sync.hubId) { mutableStateOf(sync.hubId == null) }
    var showManualSetup by remember(sync.hubId) { mutableStateOf(sync.hubId != null) }

    val rows = buildList<@Composable (Shape) -> Unit> {
        if (!changing) {
            add { shape ->
                SettingsItem(
                    title = stringResource(Res.string.tools_hub_connected),
                    shape = shape,
                    supporting = stringResource(if (sync.relay != null) Res.string.tools_hub_own_relay else Res.string.tools_hub_n0_relay),
                    icon = Res.drawable.ic_sync,
                    trailing = {
                        FilledTonalButton(
                            onClick = { onEvent(ToolsEvent.OnSyncHub("")) },
                            enabled = !state.isWorking,
                            shape = RoundedCornerShape(Dimens.RadiusFull),
                            colors = ButtonDefaults.filledTonalButtonColors(
                                containerColor = colors.accent.copy(alpha = 0.16f),
                                contentColor = colors.accent,
                            ),
                            contentPadding = PaddingValues(horizontal = Dimens.SpaceSm, vertical = Dimens.SpaceMin),
                        ) {
                            Text(
                                text = stringResource(Res.string.tools_sync_hub),
                                style = MaterialTheme.typography.labelLarge,
                                fontWeight = FontWeight.SemiBold,
                            )
                        }
                    },
                )
            }
            sync.hubReport?.let { report ->
                add { shape -> MobileHubReportSegment(report, shape) }
            }
            add { shape ->
                SettingsItem(
                    title = stringResource(Res.string.tools_hub_change),
                    shape = shape,
                    icon = Res.drawable.ic_tune,
                    opensPage = true,
                    onClick = { changing = true },
                )
            }
        } else {
            add { shape ->
                MobileHubLinkSegment(
                    state = state,
                    shape = shape,
                    canCancel = sync.hubId != null,
                    onCancel = { changing = false },
                    onEvent = onEvent,
                )
            }
            add { shape ->
                SettingsItem(
                    title = stringResource(Res.string.mobile_sync_manual_hub),
                    shape = shape,
                    supporting = stringResource(Res.string.mobile_sync_manual_hub_hint),
                    icon = Res.drawable.ic_tune,
                    onClick = { showManualSetup = !showManualSetup },
                )
            }
            if (showManualSetup) {
                add { shape ->
                    MobileHubManualSegment(state = state, shape = shape, onEvent = onEvent)
                }
            }
            sync.hubReport?.let { report ->
                add { shape -> MobileHubReportSegment(report, shape) }
            }
        }
    }

    SettingsGroup(
        label = stringResource(Res.string.tools_hub),
        rows = rows,
    )
}

@Composable
private fun MobileHubLinkSegment(
    state: ToolsState,
    shape: Shape,
    canCancel: Boolean,
    onCancel: () -> Unit,
    onEvent: (ToolsEvent) -> Unit,
) {
    val colors = QuireTheme.colors
    var link by remember { mutableStateOf("") }
    Column(
        Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(colors.cardBackground)
            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceMd),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(
                Modifier.size(40.dp).clip(CircleShape).background(colors.accent.copy(alpha = 0.16f)),
                contentAlignment = Alignment.Center,
            ) {
                Icon(painterResource(Res.drawable.ic_sync), contentDescription = null, tint = colors.accent, modifier = Modifier.size(22.dp))
            }
            Spacer(Modifier.width(Dimens.SpaceMd))
            Column(Modifier.weight(1f)) {
                Text(stringResource(Res.string.tools_hub), style = MaterialTheme.typography.titleMedium, color = colors.fillPrimary)
                Text(stringResource(Res.string.tools_hub_hint), style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary)
            }
            if (canCancel) {
                TextButton(onClick = onCancel) {
                    Text(stringResource(Res.string.name_cancel), style = MaterialTheme.typography.labelMedium, color = colors.fillSecondary)
                }
            }
        }
        Spacer(Modifier.height(Dimens.SpaceSm))
        OutlinedTextField(
            value = link,
            onValueChange = { link = it },
            placeholder = { Text(stringResource(Res.string.tools_hub_link), color = colors.fillTertiary) },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(
                onDone = {
                    if (link.isNotBlank() && !state.isWorking) {
                        onEvent(ToolsEvent.OnUseHubLink(link))
                        link = ""
                    }
                },
            ),
            modifier = Modifier.fillMaxWidth(),
        )
        Spacer(Modifier.height(Dimens.SpaceXs))
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
        ) {
            Text(
                text = stringResource(Res.string.tools_hub_link_hint),
                style = MaterialTheme.typography.bodySmall,
                color = colors.fillSecondary,
                modifier = Modifier.weight(1f),
            )
            Button(
                onClick = {
                    onEvent(ToolsEvent.OnUseHubLink(link))
                    link = ""
                },
                enabled = link.isNotBlank() && !state.isWorking,
                shape = RoundedCornerShape(Dimens.RadiusFull),
                colors = ButtonDefaults.buttonColors(containerColor = colors.accent),
            ) {
                Text(stringResource(Res.string.tools_hub_join), style = MaterialTheme.typography.labelLarge)
            }
        }
    }
}

@Composable
private fun MobileHubManualSegment(
    state: ToolsState,
    shape: Shape,
    onEvent: (ToolsEvent) -> Unit,
) {
    val sync = state.sync
    val colors = QuireTheme.colors
    var hubId by remember(sync.hubId) { mutableStateOf(sync.hubId.orEmpty()) }
    var relay by remember(sync.relay) { mutableStateOf(sync.relay.orEmpty()) }
    var code by remember { mutableStateOf("") }
    val changed = hubId.trim() != sync.hubId.orEmpty() || relay.trim() != sync.relay.orEmpty()

    Column(
        Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(colors.cardBackground)
            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceMd),
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
    ) {
        OutlinedTextField(
            value = hubId,
            onValueChange = { hubId = it },
            label = { Text(stringResource(Res.string.tools_hub_id)) },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = relay,
            onValueChange = { relay = it },
            label = { Text(stringResource(Res.string.tools_relay)) },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
            modifier = Modifier.fillMaxWidth(),
        )
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceSm),
        ) {
            Text(
                text = stringResource(Res.string.tools_relay_hint),
                style = MaterialTheme.typography.bodySmall,
                color = colors.fillSecondary,
                modifier = Modifier.weight(1f),
            )
            FilledTonalButton(
                onClick = { onEvent(ToolsEvent.OnSaveHub(hubId, relay)) },
                enabled = changed && !state.isWorking,
                shape = RoundedCornerShape(Dimens.RadiusFull),
                colors = ButtonDefaults.filledTonalButtonColors(
                    containerColor = colors.accent.copy(alpha = 0.16f),
                    contentColor = colors.accent,
                ),
            ) {
                Text(stringResource(Res.string.tools_save), style = MaterialTheme.typography.labelLarge, fontWeight = FontWeight.SemiBold)
            }
        }
        OutlinedTextField(
            value = code,
            onValueChange = { code = it },
            label = { Text(stringResource(Res.string.tools_setup_code)) },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
            modifier = Modifier.fillMaxWidth(),
        )
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.End,
        ) {
            Button(
                onClick = {
                    onEvent(ToolsEvent.OnSyncHub(code))
                    code = ""
                },
                enabled = sync.hubId != null && !changed && !state.isWorking,
                shape = RoundedCornerShape(Dimens.RadiusFull),
                colors = ButtonDefaults.buttonColors(containerColor = colors.accent),
            ) {
                Text(stringResource(Res.string.tools_sync_hub), style = MaterialTheme.typography.labelLarge)
            }
        }
    }
}

@Composable
private fun MobileHubReportSegment(report: HubReport, shape: Shape) {
    val colors = QuireTheme.colors
    Column(
        Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(colors.cardBackground)
            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceSm),
        verticalArrangement = Arrangement.spacedBy(Dimens.SpaceMin),
    ) {
        if (report.registered) {
            Text(stringResource(Res.string.tools_hub_registered), style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
        }
        if (report.joined) {
            Text(stringResource(Res.string.tools_hub_joined), style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
        }
        Text(
            stringResource(Res.string.tools_hub_report, report.pushed, report.applied, report.filesUploaded, report.filesDownloaded),
            style = MaterialTheme.typography.bodySmall,
            color = colors.fillSecondary,
        )
        if (report.overQuota > 0) {
            Text(
                stringResource(Res.string.tools_hub_over_quota, report.overQuota),
                style = MaterialTheme.typography.bodySmall,
                color = colors.accentRed,
            )
        }
    }
}

/**
 * This person's devices on the hub (by sealed name, with last sync time and Unlink)
 * and the recovery kit (hidden behind Show).
 */
@Composable
private fun MobileHubDevicesGroup(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    val sync = state.sync
    val colors = QuireTheme.colors
    val unnamed = stringResource(Res.string.tools_hub_unnamed)
    val thisMark = stringResource(Res.string.tools_hub_this)
    val revokedMark = stringResource(Res.string.tools_hub_revoked_mark)

    val rows = buildList<@Composable (Shape) -> Unit> {
        if (sync.hubDevices?.isEmpty() == true) {
            add { shape ->
                SettingsItem(
                    title = stringResource(Res.string.tools_hub_no_devices),
                    shape = shape,
                    icon = Res.drawable.ic_info,
                )
            }
        }
        sync.hubDevices?.forEach { device ->
            add { shape ->
                val isThis = device.device == sync.deviceId
                val name = device.name ?: unnamed
                val marks = listOfNotNull(
                    thisMark.takeIf { isThis },
                    revokedMark.takeIf { device.revoked },
                )
                val supporting = if (device.lastSession > 0) {
                    stringResource(Res.string.tools_hub_last_sync, ago(device.lastSession), device.batchesIn, device.batchesOut, device.filesIn, device.filesOut)
                } else {
                    stringResource(Res.string.tools_hub_last_seen, ago(device.lastSeen))
                }
                SettingsItem(
                    title = (listOf(name) + marks).joinToString(" · "),
                    shape = shape,
                    supporting = supporting,
                    icon = Res.drawable.ic_person,
                    enabled = !device.revoked,
                    trailing = if (!isThis && !device.revoked) {
                        {
                            TextButton(
                                onClick = { onEvent(ToolsEvent.OnConfirm(ToolsConfirm.RevokeHubDevice(device.device, name))) },
                                enabled = !state.isWorking,
                            ) {
                                Text(
                                    text = stringResource(Res.string.tools_revoke),
                                    style = MaterialTheme.typography.labelLarge,
                                    fontWeight = FontWeight.SemiBold,
                                    color = colors.accentRed,
                                )
                            }
                        }
                    } else {
                        null
                    },
                )
            }
        }
        add { shape ->
            SettingsItem(
                title = stringResource(Res.string.tools_hub_load_devices),
                shape = shape,
                icon = Res.drawable.ic_sync,
                enabled = !state.isWorking,
                onClick = { onEvent(ToolsEvent.OnLoadHubDevices) },
            )
        }
        add { shape ->
            val showing = sync.recoveryKit != null
            SettingsItem(
                title = stringResource(Res.string.tools_recovery_kit),
                shape = shape,
                supporting = stringResource(Res.string.tools_recovery_kit_hint),
                icon = Res.drawable.ic_info,
                onClick = {
                    if (!showing && !state.isWorking) {
                        onEvent(ToolsEvent.OnRevealRecoveryKit)
                    } else if (showing) {
                        onEvent(ToolsEvent.OnHideRecoveryKit)
                    }
                },
                trailing = {
                    TextButton(
                        onClick = {
                            if (!showing) onEvent(ToolsEvent.OnRevealRecoveryKit) else onEvent(ToolsEvent.OnHideRecoveryKit)
                        },
                        enabled = showing || !state.isWorking,
                    ) {
                        Text(
                            text = stringResource(if (showing) Res.string.tools_hide_key else Res.string.tools_show_key),
                            style = MaterialTheme.typography.labelLarge,
                            fontWeight = FontWeight.SemiBold,
                            color = colors.accent,
                        )
                    }
                },
            )
        }
        sync.recoveryKit?.let { kit ->
            add { shape ->
                MobileCopySegment(label = stringResource(Res.string.tools_recovery_kit), value = kit, shape = shape)
            }
        }
    }

    SettingsGroup(
        label = stringResource(Res.string.tools_hub_devices),
        rows = rows,
    )
}

/** A revealed identifier or recovery kit in monospace with a Copy button. */
@Composable
private fun MobileCopySegment(label: String, value: String, shape: Shape) {
    val colors = QuireTheme.colors
    val clipboard = LocalClipboardManager.current
    Row(
        Modifier
            .fillMaxWidth()
            .clip(shape)
            .background(colors.cardBackground)
            .padding(horizontal = Dimens.SpaceMd, vertical = Dimens.SpaceSm),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(label, style = MaterialTheme.typography.labelMedium, color = colors.fillSecondary)
            Spacer(Modifier.height(2.dp))
            Text(
                value,
                style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                color = colors.fillPrimary,
            )
        }
        Spacer(Modifier.width(Dimens.SpaceSm))
        FilledTonalButton(
            onClick = { clipboard.setText(AnnotatedString(value)) },
            shape = RoundedCornerShape(Dimens.RadiusFull),
            colors = ButtonDefaults.filledTonalButtonColors(
                containerColor = colors.accent.copy(alpha = 0.16f),
                contentColor = colors.accent,
            ),
            contentPadding = PaddingValues(horizontal = Dimens.SpaceSm, vertical = Dimens.SpaceMin),
        ) {
            Text(stringResource(Res.string.tools_copy), style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.SemiBold)
        }
    }
}
