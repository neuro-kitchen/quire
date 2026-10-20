package dev.neurokitchen.quire.presentation.screens.library.components

import dev.neurokitchen.quire.resources.ic_search
import dev.neurokitchen.quire.resources.name_save_search
import dev.neurokitchen.quire.resources.name_rename_search
import dev.neurokitchen.quire.resources.name_save
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.Confirm
import dev.neurokitchen.quire.presentation.screens.library.NameDialog
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.confirm_delete
import dev.neurokitchen.quire.resources.confirm_delete_project
import dev.neurokitchen.quire.resources.confirm_delete_project_body
import dev.neurokitchen.quire.resources.confirm_delete_tag
import dev.neurokitchen.quire.resources.confirm_delete_tag_body
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_sell
import dev.neurokitchen.quire.resources.name_cancel
import dev.neurokitchen.quire.resources.name_create
import dev.neurokitchen.quire.resources.name_label
import dev.neurokitchen.quire.resources.name_new_project
import dev.neurokitchen.quire.resources.name_new_project_hint
import dev.neurokitchen.quire.resources.name_new_subproject
import dev.neurokitchen.quire.resources.name_rename
import dev.neurokitchen.quire.resources.name_rename_project
import dev.neurokitchen.quire.resources.name_rename_tag
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/** Asks for a name: a new project (or subproject), or a project's or tag's new name. */
@Composable
fun NameDialogView(
    dialog: NameDialog,
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit,
) {
    val (title, initial) = when (dialog) {
        is NameDialog.NewProject -> stringResource(if (dialog.parentPath == null) Res.string.name_new_project else Res.string.name_new_subproject) to ""
        is NameDialog.RenameProject -> stringResource(Res.string.name_rename_project) to dialog.current
        is NameDialog.RenameTag -> stringResource(Res.string.name_rename_tag) to dialog.current
        is NameDialog.SaveSearch -> stringResource(Res.string.name_save_search) to ""
        is NameDialog.RenameSavedSearch -> stringResource(Res.string.name_rename_search) to dialog.current
    }
    var name by remember(dialog) { mutableStateOf(initial) }
    val canConfirm = name.isNotBlank() && name.trim() != initial
    DialogCard(onDismiss) {
        Header(
            icon = when (dialog) {
                is NameDialog.RenameTag -> Res.drawable.ic_sell
                is NameDialog.SaveSearch, is NameDialog.RenameSavedSearch -> Res.drawable.ic_search
                else -> Res.drawable.ic_create_new_folder
            },
            tint = QuireTheme.colors.accent,
            title = title,
            subtitle = when (dialog) {
                is NameDialog.NewProject -> stringResource(Res.string.name_new_project_hint)
                is NameDialog.SaveSearch -> dialog.query
                else -> null
            },
        )
        Spacer(Modifier.height(Dimens.SpaceMd))
        OutlinedTextField(
            value = name,
            onValueChange = { name = it },
            label = { Text(stringResource(Res.string.name_label)) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Spacer(Modifier.height(Dimens.SpaceLg))
        Buttons(
            confirm = stringResource(
                when (dialog) {
                    is NameDialog.NewProject -> Res.string.name_create
                    is NameDialog.SaveSearch -> Res.string.name_save
                    else -> Res.string.name_rename
                },
            ),
            confirmColor = QuireTheme.colors.accent,
            enabled = canConfirm,
            onDismiss = onDismiss,
            onConfirm = { onConfirm(name) },
        )
    }
}

/** Asks before deleting a project or a tag. */
@Composable
fun ConfirmDialog(
    confirm: Confirm,
    onDismiss: () -> Unit,
    onConfirm: () -> Unit,
) {
    val (title, body) = when (confirm) {
        is Confirm.DeleteProject -> stringResource(Res.string.confirm_delete_project, confirm.name) to stringResource(Res.string.confirm_delete_project_body)
        is Confirm.DeleteTag -> stringResource(Res.string.confirm_delete_tag, confirm.name) to stringResource(Res.string.confirm_delete_tag_body)
    }
    DialogCard(onDismiss) {
        Header(Res.drawable.ic_delete, MaterialTheme.colorScheme.error, title, body)
        Spacer(Modifier.height(Dimens.SpaceLg))
        Buttons(
            confirm = stringResource(Res.string.confirm_delete),
            confirmColor = MaterialTheme.colorScheme.error,
            enabled = true,
            onDismiss = onDismiss,
            onConfirm = onConfirm,
        )
    }
}

@Composable
private fun DialogCard(onDismiss: () -> Unit, content: @Composable () -> Unit) {
    Dialog(onDismissRequest = onDismiss) {
        Card(
            shape = RoundedCornerShape(Dimens.RadiusLarge),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainer),
            elevation = CardDefaults.cardElevation(defaultElevation = Dimens.ElevationMedium),
            modifier = Modifier.width(420.dp),
        ) {
            Column(Modifier.padding(Dimens.SpaceMd)) { content() }
        }
    }
}

@Composable
private fun Header(icon: DrawableResource, tint: Color, title: String, subtitle: String?) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Box(
            modifier = Modifier.size(36.dp).clip(RoundedCornerShape(Dimens.RadiusBase)).background(tint.copy(alpha = 0.15f)),
            contentAlignment = Alignment.Center,
        ) {
            Icon(painterResource(icon), contentDescription = null, tint = tint, modifier = Modifier.size(20.dp))
        }
        Column {
            Text(title, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.SemiBold, color = QuireTheme.colors.fillPrimary)
            if (subtitle != null) {
                Text(subtitle, style = MaterialTheme.typography.bodySmall, color = QuireTheme.colors.fillSecondary)
            }
        }
    }
}

@Composable
private fun Buttons(confirm: String, confirmColor: Color, enabled: Boolean, onDismiss: () -> Unit, onConfirm: () -> Unit) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End, verticalAlignment = Alignment.CenterVertically) {
        TextButton(onClick = onDismiss) { Text(stringResource(Res.string.name_cancel)) }
        Spacer(Modifier.width(Dimens.SpaceXs))
        Button(onClick = onConfirm, enabled = enabled, colors = ButtonDefaults.buttonColors(containerColor = confirmColor)) { Text(confirm) }
    }
}
