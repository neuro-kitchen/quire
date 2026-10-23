package dev.neurokitchen.quire.presentation.screens.library.components

import dev.neurokitchen.quire.resources.name_save_search_query
import dev.neurokitchen.quire.resources.name_save_search_title
import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.resources.name_rename_search
import dev.neurokitchen.quire.resources.name_save
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.screens.library.Confirm
import dev.neurokitchen.quire.presentation.screens.library.NameDialog
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.confirm_delete
import dev.neurokitchen.quire.resources.confirm_delete_project
import dev.neurokitchen.quire.resources.confirm_delete_project_body
import dev.neurokitchen.quire.resources.confirm_delete_tag
import dev.neurokitchen.quire.resources.confirm_delete_tag_body
import dev.neurokitchen.quire.resources.ic_auto_fix_high
import dev.neurokitchen.quire.resources.ic_create_new_folder
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.ic_sell
import dev.neurokitchen.quire.resources.name_create
import dev.neurokitchen.quire.resources.name_label
import dev.neurokitchen.quire.resources.name_new_project
import dev.neurokitchen.quire.resources.name_new_project_hint
import dev.neurokitchen.quire.resources.name_new_subproject
import dev.neurokitchen.quire.resources.name_rename
import dev.neurokitchen.quire.resources.name_rename_project
import dev.neurokitchen.quire.resources.name_rename_tag
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
        is NameDialog.SaveSearch -> stringResource(Res.string.name_save_search_title) to ""
        is NameDialog.RenameSavedSearch -> stringResource(Res.string.name_rename_search) to dialog.current
    }
    var name by remember(dialog) { mutableStateOf(initial) }
    val canConfirm = name.isNotBlank() && name.trim() != initial
    DialogShell(
        title = title,
        subtitle = when (dialog) {
            is NameDialog.NewProject -> stringResource(Res.string.name_new_project_hint)
            is NameDialog.SaveSearch -> stringResource(Res.string.name_save_search_query, dialog.query)
            else -> null
        },
        icon = when (dialog) {
            is NameDialog.RenameTag -> Res.drawable.ic_sell
            is NameDialog.SaveSearch, is NameDialog.RenameSavedSearch -> Res.drawable.ic_auto_fix_high
            else -> Res.drawable.ic_create_new_folder
        },
        width = 420.dp,
        onDismiss = onDismiss,
        actions = {
            DialogCancel(onDismiss)
            DialogConfirm(
                label = stringResource(
                    when (dialog) {
                        is NameDialog.NewProject -> Res.string.name_create
                        is NameDialog.SaveSearch -> Res.string.name_save
                        else -> Res.string.name_rename
                    },
                ),
                onClick = { onConfirm(name) },
                enabled = canConfirm,
            )
        },
    ) {
        OutlinedTextField(
            value = name,
            onValueChange = { name = it },
            label = { Text(stringResource(Res.string.name_label)) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
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
    DialogShell(
        title = title,
        subtitle = body,
        icon = Res.drawable.ic_delete,
        iconTint = MaterialTheme.colorScheme.error,
        width = 420.dp,
        onDismiss = onDismiss,
        actions = {
            DialogCancel(onDismiss)
            DialogConfirm(stringResource(Res.string.confirm_delete), onConfirm, destructive = true)
        },
    ) {}
}
