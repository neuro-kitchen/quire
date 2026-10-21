package dev.neurokitchen.quire.presentation.screens.settings.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.domain.model.SettingKind
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.designsystem.components.CompactTextField
import dev.neurokitchen.quire.presentation.screens.settings.SettingField
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.settings_apply
import dev.neurokitchen.quire.resources.settings_default
import dev.neurokitchen.quire.resources.settings_reset
import org.jetbrains.compose.resources.stringResource

/**
 * One setting: its description and key, and an editor for its kind. Text-like values apply with
 * Enter or the Apply button; booleans and choices apply as soon as they change.
 */
@Composable
fun SettingRow(
    field: SettingField,
    onDraftChange: (String) -> Unit,
    onApply: () -> Unit,
    onReset: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    Column(modifier.fillMaxWidth().padding(vertical = Dimens.SpaceXs)) {
        Text(field.help.replaceFirstChar { it.uppercase() }, style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
        Text(
            field.key,
            style = MaterialTheme.typography.labelMedium.copy(fontFamily = FontFamily.Monospace),
            color = colors.fillSecondary,
        )
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
            modifier = Modifier.padding(top = Dimens.SpaceMin),
        ) {
            when (field.kind) {
                SettingKind.Bool -> Checkbox(
                    checked = field.draft == "true",
                    onCheckedChange = { onDraftChange(it.toString()) },
                )
                SettingKind.Choice -> Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceMin)) {
                    field.choices.forEach { choice -> Choice(choice, selected = field.draft == choice) { onDraftChange(choice) } }
                }
                else -> {
                    CompactTextField(
                        value = field.draft,
                        onValueChange = onDraftChange,
                        placeholder = placeholder(field),
                        modifier = Modifier.width(360.dp).onPreviewKeyEvent { event ->
                            if (event.key == Key.Enter && field.isDirty) { onApply(); true } else false
                        },
                    )
                    if (field.isDirty) TextButton(onClick = onApply) { Text(stringResource(Res.string.settings_apply)) }
                }
            }
            if (field.saved != null) TextButton(onClick = onReset) { Text(stringResource(Res.string.settings_reset)) }
        }
        field.error?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = colors.accentRed) }
    }
}

@Composable
private fun placeholder(field: SettingField): String = when (field.kind) {
    SettingKind.Number -> "${field.min}–${field.max}"
    SettingKind.List -> "a, b, c"
    else -> stringResource(Res.string.settings_default)
}

@Composable
private fun Choice(text: String, selected: Boolean, onClick: () -> Unit) {
    val colors = QuireTheme.colors
    Text(
        text,
        style = MaterialTheme.typography.bodyMedium,
        color = if (selected) colors.onAccent else colors.fillPrimary,
        modifier = Modifier
            .clip(RoundedCornerShape(Dimens.RadiusLarge))
            .background(if (selected) colors.accent else colors.fillQuinary)
            .clickable(onClick = onClick)
            .padding(horizontal = Dimens.SpaceXs, vertical = Dimens.SpaceMin),
    )
}

