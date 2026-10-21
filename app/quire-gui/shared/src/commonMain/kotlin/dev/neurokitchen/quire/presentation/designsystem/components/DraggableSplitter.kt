package dev.neurokitchen.quire.presentation.designsystem.components

import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.PointerIcon
import androidx.compose.ui.input.pointer.pointerHoverIcon
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dev.neurokitchen.quire.presentation.designsystem.QuireTheme

/**
 * An interactive draggable vertical splitter for resizing adjacent panels.
 * Responds to drag gestures and highlights on hover. [onDelta] gets the movement in dp (the drag
 * reports pixels: widths in dp moved faster or slower than the mouse on a scaled screen);
 * [onDragEnd] when the drag stops.
 */
@Composable
fun DraggableSplitter(
    onDelta: (Float) -> Unit,
    modifier: Modifier = Modifier,
    width: Dp = 6.dp,
    onDragEnd: () -> Unit = {},
) {
    val density = LocalDensity.current.density
    val delta by rememberUpdatedState(onDelta)
    val end by rememberUpdatedState(onDragEnd)
    val interactionSource = remember { MutableInteractionSource() }
    val isHovered by interactionSource.collectIsHoveredAsState()
    val colors = QuireTheme.colors

    Box(
        modifier = modifier
            .fillMaxHeight()
            .width(width)
            .hoverable(interactionSource)
            .pointerHoverIcon(PointerIcon.Hand)
            .pointerInput(Unit) {
                detectDragGestures(onDragEnd = { end() }) { change, dragAmount ->
                    change.consume()
                    delta(dragAmount.x / density)
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        Box(
            modifier = Modifier
                .fillMaxHeight()
                .width(if (isHovered) 2.dp else 1.dp)
                .background(if (isHovered) colors.accent else colors.paneDivider),
        )
    }
}
