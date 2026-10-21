package dev.neurokitchen.quire.presentation.designsystem.components

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier

/** The least share of the row a column keeps. */
const val MIN_COLUMN_FRACTION = 0.15f

/**
 * Columns side by side, each taking its share of [fractions], with a splitter between each two.
 * A drag moves the boundary by its share of the row's real width, so it follows the mouse at any
 * window size; [onResize] gets the new fractions as the drag goes, [onResized] when it stops.
 */
@Composable
fun ResizableRow(
    fractions: List<Float>,
    onResize: (List<Float>) -> Unit,
    modifier: Modifier = Modifier,
    onResized: () -> Unit = {},
    column: @Composable (index: Int) -> Unit,
) {
    val current by rememberUpdatedState(fractions)
    BoxWithConstraints(modifier.fillMaxSize()) {
        val width = maxWidth.value
        Row(Modifier.fillMaxSize()) {
            fractions.forEachIndexed { index, fraction ->
                if (index > 0) {
                    DraggableSplitter(
                        onDelta = { delta -> onResize(resized(current, index - 1, delta, width, MIN_COLUMN_FRACTION)) },
                        onDragEnd = onResized,
                    )
                }
                Box(Modifier.weight(fraction).fillMaxHeight()) { column(index) }
            }
        }
    }
}

/**
 * [fractions] with the boundary after column [boundary] moved by [delta] (dp) in a row [width] dp
 * wide: only the two columns beside it change, neither below [min].
 */
internal fun resized(fractions: List<Float>, boundary: Int, delta: Float, width: Float, min: Float): List<Float> {
    if (width <= 0f || boundary !in 0 until fractions.size - 1) return fractions
    val left = fractions[boundary]
    val pair = left + fractions[boundary + 1]
    val newLeft = (left + delta / width).coerceIn(min, (pair - min).coerceAtLeast(min))
    return fractions.toMutableList().apply {
        this[boundary] = newLeft
        this[boundary + 1] = pair - newLeft
    }
}

/** [saved] fractions if they fit [count] columns (each at least [min], adding up to one); else even ones. */
fun fitted(saved: List<Float>, count: Int, min: Float = MIN_COLUMN_FRACTION): List<Float> {
    val fits = saved.size == count && saved.all { it >= min - 0.0001f } && kotlin.math.abs(saved.sum() - 1f) < 0.01f
    return if (fits) saved else List(count) { 1f / count }
}
