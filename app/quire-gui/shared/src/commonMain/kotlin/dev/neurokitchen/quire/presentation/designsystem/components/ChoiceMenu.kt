package dev.neurokitchen.quire.presentation.designsystem.components

import androidx.compose.foundation.layout.width
import androidx.compose.ui.unit.Dp
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_expand_more
import org.jetbrains.compose.resources.painterResource

/** A labelled button showing the choice made, opening the [choices] (value, shown name, indent). */
@Composable
fun <T> ChoiceMenu(
    label: String,
    selected: T,
    choices: List<Choice<T>>,
    onSelect: (T) -> Unit,
    modifier: Modifier = Modifier,
    /** The label's width, so menus one under another line up. */
    labelWidth: Dp? = null,
) {
    var open by remember { mutableStateOf(false) }
    Row(modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
        Text(
            label,
            style = MaterialTheme.typography.labelMedium,
            color = QuireTheme.colors.fillSecondary,
            modifier = if (labelWidth != null) Modifier.width(labelWidth) else Modifier,
        )
        Box {
            OutlinedButton(onClick = { open = true }) {
                Text(choices.firstOrNull { it.value == selected }?.name.orEmpty(), maxLines = 1)
                Icon(painterResource(Res.drawable.ic_expand_more), contentDescription = null, modifier = Modifier.size(16.dp))
            }
            DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
                choices.forEach { choice ->
                    DropdownMenuItem(
                        text = { Text(choice.name, modifier = Modifier.padding(start = Dimens.SpaceSm * choice.depth)) },
                        onClick = { open = false; onSelect(choice.value) },
                    )
                }
            }
        }
    }
}

data class Choice<T>(val value: T, val name: String, val depth: Int = 0)

/** "No project" (or "Whole library") and every project, nested ones indented. */
fun projectChoices(none: String, projects: List<Pair<String, String>>): List<Choice<String?>> =
    listOf(Choice<String?>(null, none)) + projects.map { (path, name) -> Choice<String?>(path, name, path.count { it == '/' }) }
