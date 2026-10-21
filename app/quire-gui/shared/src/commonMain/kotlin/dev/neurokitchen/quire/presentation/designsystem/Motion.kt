package dev.neurokitchen.quire.presentation.designsystem

import androidx.compose.animation.core.AnimationSpec
import androidx.compose.animation.core.FastOutLinearInEasing
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.SpringSpec
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween

object ExpressiveSprings {
    /** Instantaneous response on touch down (0-10ms) */
    fun <T> immediate(): AnimationSpec<T> = tween(
        durationMillis = 10,
        easing = FastOutLinearInEasing,
    )

    /** Easy-off natural release transition back to normal state with smooth decay */
    fun <T> easyOff(): SpringSpec<T> = spring(
        stiffness = 400f,
        dampingRatio = Spring.DampingRatioLowBouncy,
    )

    /** Snappy spring for quick UI state switches, tabs, and chip toggles (~60-80ms) */
    fun <T> quick(): SpringSpec<T> = spring(
        stiffness = Spring.StiffnessHigh,
        dampingRatio = Spring.DampingRatioNoBouncy,
    )

    /** Smooth spring for screen transitions, sheets, and dialog entries */
    fun <T> smooth(): SpringSpec<T> = spring(
        stiffness = Spring.StiffnessMedium,
        dampingRatio = Spring.DampingRatioNoBouncy,
    )

    /** Playful expressive spring with distinct bounce for hero button actions */
    fun <T> bouncy(): SpringSpec<T> = spring(
        stiffness = Spring.StiffnessMediumLow,
        dampingRatio = Spring.DampingRatioMediumBouncy,
    )
}
