package dev.neurokitchen.quire.presentation.screens.reader.components

import dev.neurokitchen.quire.resources.notes_collapse_notes
import dev.neurokitchen.quire.resources.common_link_paper
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.runtime.remember
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
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
import androidx.compose.foundation.layout.width
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
import dev.neurokitchen.quire.resources.ic_link

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
    onWikilinkClick: (String) -> Unit = {},
) {
    val colors = QuireTheme.colors
    val shape = RoundedCornerShape(Dimens.RadiusBase)
    val wikilinkRegex = remember { Regex("\\[\\[([a-zA-Z0-9_:-]+)\\]\\]") }
    Column(modifier.fillMaxHeight().background(colors.sidepane).padding(Dimens.SpaceSm)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().padding(bottom = Dimens.SpaceMin),
        ) {
            Text(stringResource(Res.string.reader_notes), style = MaterialTheme.typography.titleMedium, color = colors.fillPrimary)
            Spacer(Modifier.weight(1f))
            if (onCollapse != null) {
                IconButton(onClick = onCollapse, modifier = Modifier.size(24.dp)) {
                    Icon(
                        painterResource(Res.drawable.ic_close),
                        contentDescription = stringResource(Res.string.notes_collapse_notes),
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

        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth().padding(top = Dimens.SpaceXs),
        ) {
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(Dimens.RadiusSmall))
                    .background(colors.accent.copy(alpha = 0.12f))
                    .clickable {
                        val next = if (draft.isEmpty() || draft.endsWith(" ")) draft + "[[citeKey]]" else "$draft [[citeKey]]"
                        onDraftChange(next)
                    }
                    .padding(horizontal = Dimens.SpaceXs, vertical = Dimens.SpaceMin),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(painterResource(Res.drawable.ic_link), null, tint = colors.accent, modifier = Modifier.size(12.dp))
                    Spacer(Modifier.width(Dimens.SpaceMin))
                    Text(stringResource(Res.string.common_link_paper), style = MaterialTheme.typography.labelSmall.copy(fontSize = 11.sp, fontWeight = FontWeight.SemiBold), color = colors.accent)
                }
            }
            Spacer(Modifier.weight(1f))
            Button(
                onClick = onAdd,
                enabled = draft.isNotBlank(),
                shape = RoundedCornerShape(Dimens.RadiusSmall),
                colors = ButtonDefaults.buttonColors(containerColor = colors.accent),
            ) {
                Text(stringResource(Res.string.reader_add_note), style = MaterialTheme.typography.labelMedium)
            }
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
                        val annotated = remember(note.body, colors.accent) {
                            buildAnnotatedString {
                                var lastIdx = 0
                                wikilinkRegex.findAll(note.body).forEach { match ->
                                    append(note.body.substring(lastIdx, match.range.first))
                                    pushStyle(SpanStyle(color = colors.accent, fontWeight = FontWeight.Bold))
                                    append(match.value)
                                    pop()
                                    lastIdx = match.range.last + 1
                                }
                                if (lastIdx < note.body.length) {
                                    append(note.body.substring(lastIdx))
                                }
                            }
                        }
                        Text(annotated, style = MaterialTheme.typography.bodyMedium, color = colors.fillPrimary)

                        // Extracted Wikilinks Badges
                        val links = remember(note.body) {
                            wikilinkRegex.findAll(note.body).map { it.groupValues[1] }.toList()
                        }
                        if (links.isNotEmpty()) {
                            Spacer(Modifier.height(Dimens.SpaceMin))
                            @OptIn(ExperimentalLayoutApi::class)
                            FlowRow(horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceMin)) {
                                links.forEach { key ->
                                    Box(
                                        modifier = Modifier
                                            .clip(RoundedCornerShape(Dimens.RadiusSmall))
                                            .background(colors.accent.copy(alpha = 0.15f))
                                            .clickable { onWikilinkClick(key) }
                                            .padding(horizontal = 6.dp, vertical = 2.dp),
                                    ) {
                                        Text(
                                            "[[@$key]]",
                                            style = MaterialTheme.typography.labelSmall.copy(fontSize = 10.sp, fontWeight = FontWeight.SemiBold),
                                            color = colors.accent,
                                        )
                                    }
                                }
                            }
                        }

                        Spacer(Modifier.height(Dimens.SpaceMin))
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
