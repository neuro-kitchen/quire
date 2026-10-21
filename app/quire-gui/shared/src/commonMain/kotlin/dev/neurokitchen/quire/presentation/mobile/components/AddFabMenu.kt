package dev.neurokitchen.quire.presentation.mobile.components

import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.Dimens
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme
import dev.neurokitchen.quire.presentation.screens.library.AddTab
import dev.neurokitchen.quire.resources.Res
import dev.neurokitchen.quire.resources.add_menu_bib
import dev.neurokitchen.quire.resources.add_menu_identifier
import dev.neurokitchen.quire.resources.add_menu_manual
import dev.neurokitchen.quire.resources.add_menu_paste
import dev.neurokitchen.quire.resources.add_menu_pdf
import dev.neurokitchen.quire.resources.add_title
import dev.neurokitchen.quire.resources.common_close
import dev.neurokitchen.quire.resources.ic_add
import dev.neurokitchen.quire.resources.ic_description
import dev.neurokitchen.quire.resources.ic_draft
import dev.neurokitchen.quire.resources.ic_link
import dev.neurokitchen.quire.resources.ic_picture_as_pdf
import dev.neurokitchen.quire.resources.ic_subject
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.resources.stringResource

/** The ways in the Add menu offers, most used first, each opening Add on its tab. */
private val AddWays: List<Triple<AddTab, StringResource, DrawableResource>> = listOf(
    Triple(AddTab.Identifier, Res.string.add_menu_identifier, Res.drawable.ic_link),
    Triple(AddTab.Pdf, Res.string.add_menu_pdf, Res.drawable.ic_picture_as_pdf),
    Triple(AddTab.File, Res.string.add_menu_bib, Res.drawable.ic_description),
    Triple(AddTab.Bibtex, Res.string.add_menu_paste, Res.drawable.ic_subject),
    Triple(AddTab.Manual, Res.string.add_menu_manual, Res.drawable.ic_draft),
)

/**
 * Add, by progressive disclosure: one button; tapped, it shows every way in (Material 3
 * Expressive's FAB menu: the ways above it, the button turning into a close). The page behind
 * dims; a tap on it, or Back, closes the menu. [bottom]: the space the navigation bar takes.
 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun AddFabMenu(onAdd: (AddTab) -> Unit, bottom: Dp, modifier: Modifier = Modifier) {
    val colors = QuireTheme.colors
    var open by rememberSaveable { mutableStateOf(false) }
    val turn by animateFloatAsState(if (open) 45f else 0f)
    BackHandler(enabled = open) { open = false }
    Box(modifier.fillMaxSize()) {
        AnimatedVisibility(visible = open, enter = fadeIn(), exit = fadeOut()) {
            Box(
                Modifier
                    .fillMaxSize()
                    .background(Color.Black.copy(alpha = 0.32f))
                    .clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) { open = false },
            )
        }
        Column(
            Modifier.align(Alignment.BottomEnd).padding(end = Dimens.SpaceMd, bottom = bottom + Dimens.SpaceMd),
            horizontalAlignment = Alignment.End,
            verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
        ) {
            AnimatedVisibility(visible = open, enter = fadeIn() + scaleIn(transformOrigin = TransformOrigin(1f, 1f)), exit = fadeOut() + scaleOut(transformOrigin = TransformOrigin(1f, 1f))) {
                Column(horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(Dimens.SpaceXs)) {
                    AddWays.forEach { (tab, label, icon) ->
                        // Flat pills in one tonal colour (Expressive's FAB menu items).
                        Surface(
                            onClick = {
                                open = false
                                onAdd(tab)
                            },
                            shape = CircleShape,
                            color = colors.surfaceContainerHigh,
                            contentColor = colors.fillPrimary,
                        ) {
                            Row(
                                Modifier.height(48.dp).padding(horizontal = Dimens.SpaceMd),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(Dimens.SpaceXs),
                            ) {
                                Icon(painterResource(icon), contentDescription = null, tint = colors.accent, modifier = Modifier.size(20.dp))
                                Text(stringResource(label), style = MaterialTheme.typography.labelLarge)
                            }
                        }
                    }
                }
            }
            val label = stringResource(if (open) Res.string.common_close else Res.string.add_title)
            FloatingActionButton(
                onClick = { open = !open },
                containerColor = colors.accent,
                contentColor = colors.onAccent,
                modifier = Modifier.size(64.dp).semantics { contentDescription = label },
            ) {
                Icon(painterResource(Res.drawable.ic_add), contentDescription = null, modifier = Modifier.rotate(turn))
            }
        }
    }
}
