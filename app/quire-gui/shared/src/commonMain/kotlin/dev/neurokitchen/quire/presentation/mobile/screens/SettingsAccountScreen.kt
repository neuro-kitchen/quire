package dev.neurokitchen.quire.presentation.mobile.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.neurokitchen.quire.domain.model.SettingKind
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.DialogCancel
import dev.neurokitchen.quire.presentation.designsystem.components.DialogConfirm
import dev.neurokitchen.quire.presentation.designsystem.components.DialogShell
import dev.neurokitchen.quire.presentation.screens.settings.SettingField
import dev.neurokitchen.quire.presentation.screens.settings.SettingsEvent
import dev.neurokitchen.quire.presentation.screens.settings.SettingsState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_edit
import dev.neurokitchen.quire.resources.ic_person
import dev.neurokitchen.quire.resources.mobile_settings_account_email
import dev.neurokitchen.quire.resources.mobile_settings_account_email_hint
import dev.neurokitchen.quire.resources.mobile_settings_account_email_placeholder
import dev.neurokitchen.quire.resources.mobile_settings_account_name
import dev.neurokitchen.quire.resources.mobile_settings_account_name_hint
import dev.neurokitchen.quire.resources.mobile_settings_account_name_placeholder
import dev.neurokitchen.quire.resources.settings_not_set
import dev.neurokitchen.quire.resources.settings_reset
import dev.neurokitchen.quire.resources.settings_save
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/**
 * WhatsApp Profile style Account screen:
 * - Large centered avatar circle at the top
 * - Clean tap-to-edit rows for Name and Contact Email with explanatory footnotes
 * - Focused edit dialog when a row is tapped
 */
@Composable
internal fun AccountPage(state: SettingsState, onEvent: (SettingsEvent) -> Unit) {
    val colors = QuireTheme.colors
    val fields = state.fields
        .filterNot { it.key.contains('*') || it.kind == SettingKind.Path }
        .sortedBy {
            when (it.key) {
                "user.name" -> 0
                "user.email" -> 1
                else -> 2
            }
        }

    var editingKey by rememberSaveable { mutableStateOf<String?>(null) }
    val editingField = fields.firstOrNull { it.key == editingKey }

    if (editingField != null) {
        val meta = accountFieldMeta(editingField)
        EditAccountFieldDialog(
            field = editingField,
            meta = meta,
            onDraftChange = { onEvent(SettingsEvent.OnDraftChange(editingField.key, it)) },
            onSave = {
                onEvent(SettingsEvent.OnApply(editingField.key))
                editingKey = null
            },
            onReset = if (!editingField.saved.isNullOrBlank()) {
                {
                    onEvent(SettingsEvent.OnReset(editingField.key))
                    editingKey = null
                }
            } else {
                null
            },
            onDismiss = { editingKey = null },
        )
    }

    val savedName = fields.firstOrNull { it.key == "user.name" }?.saved?.takeIf { it.isNotBlank() }
    val initials = remember(savedName) {
        savedName?.trim()
            ?.split(Regex("\\s+"))
            ?.filter { it.isNotEmpty() }
            ?.take(2)
            ?.joinToString("") { it.first().uppercase() }
            .orEmpty()
    }

    Column(modifier = Modifier.fillMaxWidth()) {
        // Centered WhatsApp Profile Avatar
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 24.dp),
            contentAlignment = Alignment.Center,
        ) {
            Box(
                modifier = Modifier
                    .size(96.dp)
                    .clip(CircleShape)
                    .background(colors.accent.copy(alpha = 0.16f)),
                contentAlignment = Alignment.Center,
            ) {
                if (initials.isNotEmpty()) {
                    Text(
                        text = initials,
                        style = MaterialTheme.typography.headlineMedium.copy(
                            fontWeight = FontWeight.Bold,
                        ),
                        color = colors.accent,
                    )
                } else {
                    Icon(
                        painter = painterResource(Res.drawable.ic_person),
                        contentDescription = null,
                        tint = colors.accent,
                        modifier = Modifier.size(44.dp),
                    )
                }
            }
        }

        fields.forEachIndexed { index, field ->
            val meta = accountFieldMeta(field)
            val valueText = field.saved?.takeIf { it.isNotBlank() }
                ?: stringResource(Res.string.settings_not_set)
            val isUnset = field.saved.isNullOrBlank()

            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { editingKey = field.key }
                    .padding(horizontal = 24.dp, vertical = 16.dp),
                verticalAlignment = Alignment.Top,
            ) {
                Icon(
                    painter = painterResource(meta.icon),
                    contentDescription = null,
                    tint = colors.fillSecondary,
                    modifier = Modifier.padding(top = 2.dp).size(22.dp),
                )
                Spacer(Modifier.width(22.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = meta.title,
                                style = MaterialTheme.typography.bodySmall.copy(fontSize = 12.sp),
                                color = colors.fillSecondary,
                            )
                            Spacer(Modifier.height(2.dp))
                            Text(
                                text = valueText,
                                style = MaterialTheme.typography.bodyLarge.copy(
                                    fontSize = 16.sp,
                                    fontWeight = if (isUnset) FontWeight.Normal else FontWeight.Medium,
                                ),
                                color = if (isUnset) colors.fillTertiary else colors.fillPrimary,
                            )
                        }
                        Icon(
                            painter = painterResource(Res.drawable.ic_edit),
                            contentDescription = meta.title,
                            tint = colors.accent,
                            modifier = Modifier.size(20.dp),
                        )
                    }
                    if (!meta.supporting.isNullOrBlank()) {
                        Spacer(Modifier.height(6.dp))
                        Text(
                            text = meta.supporting,
                            style = MaterialTheme.typography.bodySmall.copy(
                                fontSize = 12.sp,
                                lineHeight = 17.sp,
                            ),
                            color = colors.fillSecondary,
                        )
                    }
                }
            }

            if (index < fields.lastIndex) {
                HorizontalDivider(
                    color = colors.border.copy(alpha = 0.3f),
                    modifier = Modifier.padding(start = 68.dp),
                )
            }
        }
    }
}

@Composable
private fun accountFieldMeta(field: SettingField): AccountFieldMeta = when (field.key) {
    "user.name" -> AccountFieldMeta(
        title = stringResource(Res.string.mobile_settings_account_name),
        supporting = stringResource(Res.string.mobile_settings_account_name_hint),
        placeholder = stringResource(Res.string.mobile_settings_account_name_placeholder),
        icon = Res.drawable.ic_person,
    )
    "user.email" -> AccountFieldMeta(
        title = stringResource(Res.string.mobile_settings_account_email),
        supporting = stringResource(Res.string.mobile_settings_account_email_hint),
        placeholder = stringResource(Res.string.mobile_settings_account_email_placeholder),
        icon = Res.drawable.ic_description,
    )
    else -> AccountFieldMeta(
        title = field.help.replaceFirstChar { it.uppercase() },
        supporting = null,
        placeholder = "",
        icon = Res.drawable.ic_person,
    )
}

@Composable
private fun EditAccountFieldDialog(
    field: SettingField,
    meta: AccountFieldMeta,
    onDraftChange: (String) -> Unit,
    onSave: () -> Unit,
    onReset: (() -> Unit)?,
    onDismiss: () -> Unit,
) {
    val colors = QuireTheme.colors
    DialogShell(
        title = meta.title,
        subtitle = meta.supporting,
        icon = meta.icon,
        onDismiss = onDismiss,
        modifier = Modifier.padding(horizontal = Dimens.SpaceMd),
        actions = {
            if (onReset != null) {
                TextButton(onClick = onReset) {
                    Text(
                        text = stringResource(Res.string.settings_reset),
                        color = colors.fillSecondary,
                    )
                }
            }
            DialogCancel(onClick = onDismiss)
            DialogConfirm(
                label = stringResource(Res.string.settings_save),
                onClick = onSave,
            )
        },
    ) {
        OutlinedTextField(
            value = field.draft,
            onValueChange = onDraftChange,
            placeholder = if (meta.placeholder.isNotEmpty()) {
                { Text(meta.placeholder, color = colors.fillTertiary) }
            } else {
                null
            },
            isError = field.error != null,
            supportingText = field.error?.let { { Text(it) } },
            singleLine = true,
            shape = RoundedCornerShape(Dimens.RadiusBase),
            keyboardOptions = KeyboardOptions(
                keyboardType = when (field.kind) {
                    SettingKind.Email -> KeyboardType.Email
                    SettingKind.Number -> KeyboardType.Number
                    else -> KeyboardType.Text
                },
                imeAction = ImeAction.Done,
            ),
            keyboardActions = KeyboardActions(onDone = { onSave() }),
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

private data class AccountFieldMeta(
    val title: String,
    val supporting: String?,
    val placeholder: String,
    val icon: DrawableResource,
)
