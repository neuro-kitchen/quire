package dev.neurokitchen.quire.presentation.screens.reader.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_delete
import dev.neurokitchen.quire.resources.reader_add_note
import dev.neurokitchen.quire.resources.reader_no_notes
import dev.neurokitchen.quire.resources.reader_note_hint
import dev.neurokitchen.quire.resources.reader_notes
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

import dev.neurokitchen.quire.resources.ic_close

/** The paper's notes beside the reader, and a field to write a new one. */
@Composable
fun NotesPanel(
    notes: List<Note>,
    draft: String,
    onDraftChange: (String) -> Unit,
    onAdd: () -> Unit,
    onRemove: (String) -> Unit,
    modifier: Modifier = Modifier,
    onCollapse: (() -> Unit)? = null,
) {
    val colors = QuireTheme.colors
    val shape = RoundedCornerShape(Dimens.RadiusBase)
    Column(modifier.fillMaxHeight().background(colors.sidepane).padding(Dimens.SpaceSm)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().padding(bottom = 4.dp),
        ) {
            Text(stringResource(Res.string.reader_notes), style = MaterialTheme.typography.titleMedium, color = colors.fillPrimary)
            Spacer(Modifier.weight(1f))
            if (onCollapse != null) {
                IconButton(onClick = onCollapse, modifier = Modifier.size(24.dp)) {
                    Icon(
                        painterResource(Res.drawable.ic_close),
                        contentDescription = "Collapse Notes",
                        tint = colors.fillSecondary,
                        modifier = Modifier.size(16.dp),
                    )
                }
            }
        }

        BasicTextField(
            value = draft,
            onValueChange = onDraftChange,
            textStyle = MaterialTheme.typography.bodyMedium.copy(color = colors.fillPrimary),
            cursorBrush = SolidColor(colors.accent),
            modifier = Modifier
                .padding(top = Dimens.SpaceXs)
                .fillMaxWidth()
                .heightIn(min = 72.dp)
                .clip(shape)
                .background(colors.background)
                .border(1.dp, colors.border, shape)
                .padding(Dimens.SpaceSm),
            decorationBox = { inner ->
                if (draft.isEmpty()) Text(stringResource(Res.string.reader_note_hint), style = MaterialTheme.typography.bodyMedium, color = colors.fillTertiary)
                inner()
            },
        )

        Button(
            onClick = onAdd,
            enabled = draft.isNotBlank(),
            shape = RoundedCornerShape(Dimens.RadiusSmall),
            colors = ButtonDefaults.buttonColors(containerColor = colors.accent),
            modifier = Modifier.padding(vertical = Dimens.SpaceXs).align(Alignment.End),
        ) {
            Text(stringResource(Res.string.reader_add_note), style = MaterialTheme.typography.labelMedium)
        }

        if (notes.isEmpty()) {
            Text(stringResource(Res.string.reader_no_notes), style = MaterialTheme.typography.bodyMedium, color = colors.fillSecondary)
        }

        LazyColumn(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            items(notes, key = { it.id }) { note ->
                Card(
                    shape = RoundedCornerShape(Dimens.RadiusBase),
                    colors = CardDefaults.cardColors(containerColor = colors.background),
                    modifier = Modifier.fillMaxWidth().border(1.dp, colors.border, RoundedCornerShape(Dimens.RadiusBase)),
                ) {
                    Column(Modifier.padding(Dimens.SpaceSm)) {
                        Text(note.body, style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)
                        Spacer(Modifier.height(4.dp))
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier.fillMaxWidth(),
                        ) {
                            Text(note.updated, style = MaterialTheme.typography.labelSmall, color = colors.fillTertiary)
                            Spacer(Modifier.weight(1f))
                            IconButton(
                                onClick = { onRemove(note.id) },
                                modifier = Modifier.size(24.dp),
                            ) {
                                Icon(painterResource(Res.drawable.ic_delete), null, tint = colors.fillSecondary, modifier = Modifier.size(14.dp))
                            }
                        }
                    }
                }
            }
        }
    }
}
