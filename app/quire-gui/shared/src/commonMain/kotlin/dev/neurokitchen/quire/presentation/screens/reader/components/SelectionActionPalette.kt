package dev.neurokitchen.quire.presentation.screens.reader.components

import dev.neurokitchen.quire.resources.selection_copy_with_citation
import dev.neurokitchen.quire.resources.selection_copy_text
import dev.neurokitchen.quire.resources.selection_add_note
import org.jetbrains.compose.resources.stringResource
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.ic_content_copy
import dev.neurokitchen.quire.resources.ic_sticky_note_2
import dev.neurokitchen.quire.resources.ic_subject
import org.jetbrains.compose.resources.painterResource

@Composable
fun SelectionActionPalette(
    visible: Boolean,
    onHighlight: (color: String) -> Unit,
    onAddNote: () -> Unit,
    onCopy: () -> Unit,
    onCopyCitation: () -> Unit,
    modifier: Modifier = Modifier,
) {
    AnimatedVisibility(
        visible = visible,
        enter = fadeIn() + scaleIn(initialScale = 0.9f),
        exit = fadeOut() + scaleOut(targetScale = 0.9f),
        modifier = modifier,
    ) {
        val colors = QuireTheme.colors
        Card(
            shape = RoundedCornerShape(Dimens.RadiusFull),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHighest),
            elevation = CardDefaults.cardElevation(defaultElevation = Dimens.ElevationMedium),
            modifier = Modifier
                .shadow(8.dp, RoundedCornerShape(Dimens.RadiusFull))
                .border(1.dp, colors.fillQuarternary, RoundedCornerShape(Dimens.RadiusFull)),
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceMin),
                modifier = Modifier.padding(horizontal = Dimens.SpaceXs, vertical = Dimens.SpaceMin),
            ) {
                // One colour: the engine keeps no colour per highlight yet, so others would not stay.
                val paletteColors = listOf("yellow" to colors.highlightYellow)

                for ((name, color) in paletteColors) {
                    Box(
                        modifier = Modifier
                            .size(22.dp)
                            .clip(CircleShape)
                            .background(color)
                            .border(1.dp, Color.Black.copy(alpha = 0.15f), CircleShape)
                            .clickable { onHighlight(name) },
                    )
                }

                Spacer(Modifier.width(Dimens.SpaceMin))
                VerticalDivider(color = colors.fillQuarternary, modifier = Modifier.height(20.dp))
                Spacer(Modifier.width(Dimens.SpaceMin))

                // Add note to selection
                IconButton(
                    onClick = onAddNote,
                    modifier = Modifier.size(28.dp),
                ) {
                    Icon(
                        painterResource(Res.drawable.ic_sticky_note_2),
                        contentDescription = stringResource(Res.string.selection_add_note),
                        tint = colors.sectionNotes,
                        modifier = Modifier.size(16.dp),
                    )
                }

                // Copy plain text
                IconButton(
                    onClick = onCopy,
                    modifier = Modifier.size(28.dp),
                ) {
                    Icon(
                        painterResource(Res.drawable.ic_content_copy),
                        contentDescription = stringResource(Res.string.selection_copy_text),
                        tint = colors.fillPrimary,
                        modifier = Modifier.size(16.dp),
                    )
                }

                // Copy with citation
                IconButton(
                    onClick = onCopyCitation,
                    modifier = Modifier.size(28.dp),
                ) {
                    Icon(
                        painterResource(Res.drawable.ic_subject),
                        contentDescription = stringResource(Res.string.selection_copy_with_citation),
                        tint = colors.accent,
                        modifier = Modifier.size(16.dp),
                    )
                }
            }
        }
    }
}
