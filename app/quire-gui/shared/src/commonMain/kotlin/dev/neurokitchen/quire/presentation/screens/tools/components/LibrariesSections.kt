package dev.neurokitchen.quire.presentation.screens.tools.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.CompactTextField
import dev.neurokitchen.quire.presentation.platform.pickSaveFile
import dev.neurokitchen.quire.presentation.screens.tools.ToolsConfirm
import dev.neurokitchen.quire.presentation.screens.tools.ToolsEvent
import dev.neurokitchen.quire.presentation.screens.tools.ToolsState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.menu_remove
import dev.neurokitchen.quire.resources.tools_add_device
import dev.neurokitchen.quire.resources.tools_backup
import dev.neurokitchen.quire.resources.tools_backup_hint
import dev.neurokitchen.quire.resources.tools_choose_backup
import dev.neurokitchen.quire.resources.tools_choose_project
import dev.neurokitchen.quire.resources.tools_collaborator_id
import dev.neurokitchen.quire.resources.tools_collaborator_name
import dev.neurokitchen.quire.resources.tools_create
import dev.neurokitchen.quire.resources.tools_default
import dev.neurokitchen.quire.resources.tools_device_id
import dev.neurokitchen.quire.resources.tools_device_name
import dev.neurokitchen.quire.resources.tools_devices
import dev.neurokitchen.quire.resources.tools_devices_hint
import dev.neurokitchen.quire.resources.tools_hide_key
import dev.neurokitchen.quire.resources.tools_import_key
import dev.neurokitchen.quire.resources.tools_libraries_hint
import dev.neurokitchen.quire.resources.tools_make_default
import dev.neurokitchen.quire.resources.tools_new_library
import dev.neurokitchen.quire.resources.tools_no_devices
import dev.neurokitchen.quire.resources.tools_not_shared
import dev.neurokitchen.quire.resources.tools_open
import dev.neurokitchen.quire.resources.tools_report
import dev.neurokitchen.quire.resources.tools_role_editor
import dev.neurokitchen.quire.resources.tools_role_owner
import dev.neurokitchen.quire.resources.tools_role_reader
import dev.neurokitchen.quire.resources.tools_section_libraries
import dev.neurokitchen.quire.resources.tools_section_sync
import dev.neurokitchen.quire.resources.tools_share
import dev.neurokitchen.quire.resources.tools_sharing
import dev.neurokitchen.quire.resources.tools_show_key
import dev.neurokitchen.quire.resources.tools_sync_now
import dev.neurokitchen.quire.resources.tools_this_device
import dev.neurokitchen.quire.resources.tools_use_key
import dev.neurokitchen.quire.resources.tools_user_id
import dev.neurokitchen.quire.resources.tools_user_key
import dev.neurokitchen.quire.resources.tools_user_key_hint
import org.jetbrains.compose.resources.stringResource

/** The libraries in the config: open, make default, create; back up the open one. */
@Composable
internal fun LibrariesSection(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    SectionHeader(Res.string.tools_section_libraries, Res.string.tools_libraries_hint)
    state.libraries.forEach { library ->
        Group {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text(
                        library.name ?: library.path,
                        style = MaterialTheme.typography.bodyMedium,
                        fontWeight = if (library.isDefault) FontWeight.SemiBold else FontWeight.Normal,
                        color = QuireTheme.colors.fillPrimary,
                    )
                    Hint(listOfNotNull(library.path, if (library.isDefault) stringResource(Res.string.tools_default) else null).joinToString(" · "))
                }
                SmallAction(Res.string.tools_open, enabled = !state.isWorking) { onEvent(ToolsEvent.OnOpenLibrary(library.name)) }
                if (!library.isDefault && library.name != null) {
                    SmallAction(Res.string.tools_make_default, enabled = !state.isWorking) { onEvent(ToolsEvent.OnSetDefaultLibrary(library.name)) }
                }
            }
        }
    }
    var name by remember { mutableStateOf("") }
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        CompactTextField(value = name, onValueChange = { name = it }, placeholder = stringResource(Res.string.tools_new_library), modifier = Modifier.width(240.dp))
        SecondaryAction(Res.string.tools_create, enabled = name.isNotBlank() && !state.isWorking) {
            onEvent(ToolsEvent.OnCreateLibrary(name))
            name = ""
        }
    }
    Spacer(Modifier.height(Dimens.SpaceMd))
    val backupTitle = stringResource(Res.string.tools_choose_backup)
    Hint(stringResource(Res.string.tools_backup_hint))
    Spacer(Modifier.height(Dimens.SpaceXs))
    PrimaryAction(Res.string.tools_backup, enabled = !state.isWorking) { pickSaveFile(backupTitle, "quire-backup.tar.gz")?.let { onEvent(ToolsEvent.OnBackup(it)) } }
}

/** This device's IDs and key, the other devices to sync with, and sharing a project. */
@Composable
internal fun SyncSection(state: ToolsState, onEvent: (ToolsEvent) -> Unit) {
    val sync = state.sync
    SectionHeader(Res.string.tools_section_sync)
    Text(stringResource(Res.string.tools_this_device), style = MaterialTheme.typography.titleSmall, color = QuireTheme.colors.fillPrimary)
    Group {
        sync.deviceId?.let { CopyRow(Res.string.tools_device_id, it) }
        sync.userId?.let { CopyRow(Res.string.tools_user_id, it) }
        Spacer(Modifier.height(Dimens.SpaceXs))
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                Text(stringResource(Res.string.tools_user_key), style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary)
                Hint(stringResource(Res.string.tools_user_key_hint))
            }
            if (sync.revealedKey == null) {
                SmallAction(Res.string.tools_show_key, enabled = !state.isWorking) { onEvent(ToolsEvent.OnRevealKey) }
            } else {
                SmallAction(Res.string.tools_hide_key) { onEvent(ToolsEvent.OnHideKey) }
            }
        }
        sync.revealedKey?.let { CopyRow(Res.string.tools_user_key, it) }
        var key by remember { mutableStateOf("") }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            CompactTextField(value = key, onValueChange = { key = it }, placeholder = stringResource(Res.string.tools_import_key), modifier = Modifier.weight(1f))
            SmallAction(Res.string.tools_use_key, enabled = key.isNotBlank() && !state.isWorking) {
                onEvent(ToolsEvent.OnConfirm(ToolsConfirm.ImportKey(key)))
                key = ""
            }
        }
    }

    Spacer(Modifier.height(Dimens.SpaceSm))
    Text(stringResource(Res.string.tools_devices), style = MaterialTheme.typography.titleSmall, color = QuireTheme.colors.fillPrimary)
    Hint(stringResource(Res.string.tools_devices_hint))
    Spacer(Modifier.height(Dimens.SpaceXs))
    Group {
        if (sync.peers.isEmpty()) Hint(stringResource(Res.string.tools_no_devices))
        sync.peers.forEach { peer ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text(peer.name, style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillPrimary)
                    Text(peer.deviceId, style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace), color = QuireTheme.colors.fillSecondary, maxLines = 1)
                }
                SmallAction(Res.string.tools_sync_now, enabled = !state.isWorking) { onEvent(ToolsEvent.OnSyncWith(peer.name)) }
                SmallAction(Res.string.menu_remove, enabled = !state.isWorking) { onEvent(ToolsEvent.OnRemovePeer(peer.name)) }
            }
        }
        var name by remember { mutableStateOf("") }
        var id by remember { mutableStateOf("") }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            CompactTextField(value = name, onValueChange = { name = it }, placeholder = stringResource(Res.string.tools_device_name), modifier = Modifier.width(140.dp))
            CompactTextField(value = id, onValueChange = { id = it }, placeholder = stringResource(Res.string.tools_device_id), modifier = Modifier.weight(1f))
            SmallAction(Res.string.tools_add_device, enabled = name.isNotBlank() && id.isNotBlank() && !state.isWorking) {
                onEvent(ToolsEvent.OnAddPeer(name, id))
                name = ""
                id = ""
            }
        }
        sync.lastReport?.let { r ->
            Hint(stringResource(Res.string.tools_report, r.peer, r.sent, r.applied, r.kept, r.refused, r.filesSent, r.filesReceived))
        }
    }

    Spacer(Modifier.height(Dimens.SpaceSm))
    Text(stringResource(Res.string.tools_sharing), style = MaterialTheme.typography.titleSmall, color = QuireTheme.colors.fillPrimary)
    Spacer(Modifier.height(Dimens.SpaceXs))
    ProjectPicker(state.projects, sync.sharing, Res.string.tools_choose_project) { onEvent(ToolsEvent.OnSharingSelect(it)) }
    if (sync.sharing != null) {
        Spacer(Modifier.height(Dimens.SpaceXs))
        Group {
            if (sync.members.isEmpty()) Hint(stringResource(Res.string.tools_not_shared))
            sync.members.forEach { member ->
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(Modifier.weight(1f)) {
                        Text("${member.name} · ${roleLabel(member.role)}", style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillPrimary)
                        Text(member.userId, style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace), color = QuireTheme.colors.fillSecondary, maxLines = 1)
                    }
                    SmallAction(Res.string.menu_remove, enabled = !state.isWorking) { onEvent(ToolsEvent.OnUnshare(member.userId)) }
                }
            }
            var userId by remember { mutableStateOf("") }
            var name by remember { mutableStateOf("") }
            var role by remember { mutableStateOf("editor") }
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                CompactTextField(value = userId, onValueChange = { userId = it }, placeholder = stringResource(Res.string.tools_collaborator_id), modifier = Modifier.weight(1f))
                CompactTextField(value = name, onValueChange = { name = it }, placeholder = stringResource(Res.string.tools_collaborator_name), modifier = Modifier.width(140.dp))
                ChoicePicker(Roles, role, label = { roleLabel(it) }) { role = it }
                SmallAction(Res.string.tools_share, enabled = userId.isNotBlank() && name.isNotBlank() && !state.isWorking) {
                    onEvent(ToolsEvent.OnShare(userId, name, role))
                    userId = ""
                    name = ""
                }
            }
        }
    }
}

/** The engine's role words. */
private val Roles = listOf("reader", "editor", "owner")

@Composable
private fun roleLabel(role: String) = when (role) {
    "reader" -> stringResource(Res.string.tools_role_reader)
    "editor" -> stringResource(Res.string.tools_role_editor)
    "owner" -> stringResource(Res.string.tools_role_owner)
    else -> role
}
