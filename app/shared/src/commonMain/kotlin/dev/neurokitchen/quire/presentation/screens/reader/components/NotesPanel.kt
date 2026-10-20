package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.reader_add_note
import dev.neurokitchen.quire.resources.reader_no_notes
import dev.neurokitchen.quire.resources.reader_note_hint
import dev.neurokitchen.quire.resources.reader_notes
import dev.neurokitchen.quire.resources.reader_remove
import org.jetbrains.compose.resources.stringResource

/** The paper's notes beside the reader, and a field to write a new one. */
@Composable
fun NotesPanel(
    notes: List<Note>,
    draft: String,
    onDraftChange: (String) -> Unit,
    onAdd: () -> Unit,
    onRemove: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = QuireTheme.colors
    val shape = RoundedCornerShape(Dimens.RadiusLarge)
    Column(modifier.fillMaxHeight().background(colors.sidepane).padding(Dimens.SpaceSm)) {
        Text(stringResource(Res.string.reader_notes), style = MaterialTheme.typography.titleMedium, color = colors.fillPrimary)
        BasicTextField(
            value = draft,
            onValueChange = onDraftChange,
            textStyle = MaterialTheme.typography.bodyMedium.copy(color = colors.fillPrimary),
            cursorBrush = SolidColor(colors.accent),
            modifier = Modifier
                .padding(top = Dimens.SpaceXs)
                .fillMaxWidth()
                .heightIn(min = 72.dp)
                .background(colors.background, shape)
                .border(1.dp, colors.border, shape)
                .padding(Dimens.SpaceXs),
            decorationBox = { inner ->
                if (draft.isEmpty()) Text(stringResource(Res.string.reader_note_hint), style = MaterialTheme.typography.bodyMedium, color = colors.fillTertiary)
                inner()
            },
        )
        Button(onClick = onAdd, enabled = draft.isNotBlank(), modifier = Modifier.padding(vertical = Dimens.SpaceXs)) {
            Text(stringResource(Res.string.reader_add_note))
        }
        if (notes.isEmpty()) {
            Text(stringResource(Res.string.reader_no_notes), style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary)
        }
        LazyColumn(Modifier.fillMaxWidth()) {
            items(notes, key = { it.id }) { note ->
                Column(
                    Modifier.padding(vertical = 4.dp).fillMaxWidth().background(colors.background, shape)
                        .border(1.dp, colors.border, shape).padding(Dimens.SpaceXs),
                ) {
                    Text(note.body, style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                    Text(note.updated, style = MaterialTheme.typography.labelSmall, color = colors.fillTertiary)
                    TextButton(onClick = { onRemove(note.id) }) { Text(stringResource(Res.string.reader_remove)) }
                }
            }
        }
    }
}
