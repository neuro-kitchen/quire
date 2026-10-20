package dev.neurokitchen.quire.presentation.screens.settings.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.settings.RawConfigState
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.settings_file_ok
import dev.neurokitchen.quire.resources.settings_revert
import dev.neurokitchen.quire.resources.settings_save
import org.jetbrains.compose.resources.stringResource

/** `config.toml` as text: checked as you type, saved whole (or not at all). */
@Composable
fun RawConfigEditor(
    state: RawConfigState,
    onChange: (String) -> Unit,
    onSave: () -> Unit,
    onRevert: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    val shape = RoundedCornerShape(Dimens.RadiusLarge)
    Column(modifier.fillMaxSize()) {
        BasicTextField(
            value = state.text,
            onValueChange = onChange,
            textStyle = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace, color = colors.fillPrimary),
            cursorBrush = SolidColor(colors.accent),
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth()
                .background(colors.background, shape)
                .border(1.dp, colors.border, shape)
                .verticalScroll(rememberScrollState())
                .padding(Dimens.SpaceSm),
        )
        val problems = state.problems + listOfNotNull(state.error)
        Column(Modifier.padding(vertical = Dimens.SpaceXs)) {
            if (problems.isEmpty()) {
                Text(stringResource(Res.string.settings_file_ok), style = MaterialTheme.typography.bodySmall, color = colors.fillSecondary)
            }
            problems.forEach { Text(it, style = MaterialTheme.typography.bodySmall, color = colors.accentRed) }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
            Button(onClick = onSave, enabled = state.isDirty) { Text(stringResource(Res.string.settings_save)) }
            TextButton(onClick = onRevert, enabled = state.isDirty) { Text(stringResource(Res.string.settings_revert)) }
        }
    }
}
