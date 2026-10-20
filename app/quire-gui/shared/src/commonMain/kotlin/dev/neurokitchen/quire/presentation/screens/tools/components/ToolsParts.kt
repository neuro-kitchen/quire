package dev.neurokitchen.quire.presentation.screens.tools.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.tools.ProjectChoice
import dev.neurokitchen.quire.presentation.screens.tools.ReferenceLine
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.tools_copy
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.stringResource

/** A section's title and what it does. */
@Composable
internal fun SectionHeader(title: StringResource, hint: StringResource? = null) {
    Text(stringResource(title), style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = QuireTheme.colors.fillPrimary)
    if (hint != null) Hint(stringResource(hint))
    Spacer(Modifier.height(Dimens.SpaceSm))
}

@Composable
internal fun Hint(text: String, modifier: Modifier = Modifier) {
    Text(text, style = MaterialTheme.typography.bodySmall, color = QuireTheme.colors.fillSecondary, modifier = modifier)
}

/** The section's main action; waits while another runs. */
@Composable
internal fun PrimaryAction(label: StringResource, enabled: Boolean = true, onClick: () -> Unit) {
    Button(
        onClick = onClick,
        enabled = enabled,
        colors = ButtonDefaults.buttonColors(containerColor = QuireTheme.colors.accent),
        contentPadding = PaddingValues(horizontal = Dimens.SpaceSm),
        modifier = Modifier.height(32.dp),
    ) { Text(stringResource(label), style = MaterialTheme.typography.labelMedium) }
}

@Composable
internal fun SecondaryAction(label: StringResource, enabled: Boolean = true, onClick: () -> Unit) {
    OutlinedButton(
        onClick = onClick,
        enabled = enabled,
        contentPadding = PaddingValues(horizontal = Dimens.SpaceSm),
        modifier = Modifier.height(32.dp),
    ) { Text(stringResource(label), style = MaterialTheme.typography.labelMedium) }
}

@Composable
internal fun SmallAction(label: StringResource, enabled: Boolean = true, color: androidx.compose.ui.graphics.Color? = null, onClick: () -> Unit) {
    TextButton(onClick = onClick, enabled = enabled, contentPadding = PaddingValues(horizontal = Dimens.SpaceXs), modifier = Modifier.height(28.dp)) {
        Text(stringResource(label), style = MaterialTheme.typography.labelMedium, color = color ?: androidx.compose.ui.graphics.Color.Unspecified)
    }
}

/** A group of rows on a card. */
@Composable
internal fun Group(content: @Composable () -> Unit) {
    Card(
        shape = RoundedCornerShape(Dimens.RadiusBase),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow),
        modifier = Modifier.fillMaxWidth().padding(bottom = Dimens.SpaceSm),
    ) {
        Column(Modifier.padding(Dimens.SpaceSm)) { content() }
    }
}

@Composable
internal fun ReferenceText(reference: ReferenceLine, modifier: Modifier = Modifier) {
    Column(modifier) {
        Text(reference.title, style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillPrimary, maxLines = 2)
        Hint(listOf(reference.creatorYear, reference.citeKey).filter { it.isNotBlank() }.joinToString(" · "))
    }
}

/** A value to copy (an ID): label, the value in monospace, Copy. */
@Composable
internal fun CopyRow(label: StringResource, value: String) {
    val clipboard = LocalClipboardManager.current
    Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.fillMaxWidth()) {
        Text(stringResource(label), style = MaterialTheme.typography.bodyMedium, color = QuireTheme.colors.fillSecondary, modifier = Modifier.width(110.dp))
        Text(
            value,
            style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
            color = QuireTheme.colors.fillPrimary,
            modifier = Modifier.weight(1f),
        )
        SmallAction(Res.string.tools_copy) { clipboard.setText(AnnotatedString(value)) }
    }
}

/** Picks a project, or [none] (`null`). */
@Composable
internal fun ProjectPicker(projects: List<ProjectChoice>, selected: String?, none: StringResource, onSelect: (String?) -> Unit) {
    var open by remember { mutableStateOf(false) }
    Box {
        OutlinedButton(onClick = { open = true }, contentPadding = PaddingValues(horizontal = Dimens.SpaceSm), modifier = Modifier.height(32.dp)) {
            Text(projects.firstOrNull { it.path == selected }?.name ?: stringResource(none), style = MaterialTheme.typography.labelMedium)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            DropdownMenuItem(text = { Text(stringResource(none)) }, onClick = { open = false; onSelect(null) })
            projects.forEach { project ->
                DropdownMenuItem(
                    text = { Text("  ".repeat(project.path.count { it == '/' }) + project.name) },
                    onClick = { open = false; onSelect(project.path) },
                )
            }
        }
    }
}

/** Picks one of [choices], shown through [label]. */
@Composable
internal fun ChoicePicker(choices: List<String>, selected: String, label: @Composable (String) -> String = { it }, onSelect: (String) -> Unit) {
    var open by remember { mutableStateOf(false) }
    Box {
        OutlinedButton(onClick = { open = true }, contentPadding = PaddingValues(horizontal = Dimens.SpaceSm), modifier = Modifier.height(32.dp)) {
            Text(label(selected), style = MaterialTheme.typography.labelMedium)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            choices.forEach { choice -> DropdownMenuItem(text = { Text(label(choice)) }, onClick = { open = false; onSelect(choice) }) }
        }
    }
}
