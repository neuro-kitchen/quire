package dev.neurokitchen.quire.presentation.designsystem

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

private val LocalQuireColors = staticCompositionLocalOf { LightQuireColors }

/** Zotero's type scale: 13 px text on 20 px lines, 16 and 20 px headings. */
private val DesktopTypography = Typography().run {
    val base = TextStyle(fontSize = 13.sp, lineHeight = 20.sp)
    copy(
        bodyLarge = base,
        bodyMedium = base,
        bodySmall = base.copy(fontSize = 12.sp, lineHeight = 16.sp),
        labelLarge = base.copy(fontWeight = FontWeight.SemiBold),
        labelMedium = base.copy(fontSize = 12.sp, lineHeight = 16.sp),
        labelSmall = base.copy(fontSize = 11.sp, lineHeight = 14.sp),
        titleMedium = base.copy(fontWeight = FontWeight.SemiBold),
        titleLarge = TextStyle(fontSize = 16.sp, lineHeight = 22.sp, fontWeight = FontWeight.SemiBold),
        headlineSmall = TextStyle(fontSize = 20.sp, lineHeight = 26.sp),
    )
}

/**
 * The app's theme: light or dark following the system. `desktop` switches to the dense desktop type
 * scale; phones keep Material's.
 */
@Composable
fun QuireTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    desktop: Boolean = false,
    content: @Composable () -> Unit,
) {
    val colors = if (darkTheme) DarkQuireColors else LightQuireColors
    val scheme = if (darkTheme) {
        darkColorScheme(
            primary = colors.accent, onPrimary = colors.onAccent,
            background = colors.background, surface = colors.background,
            surfaceContainerLow = colors.sidepane, outline = colors.border,
        )
    } else {
        lightColorScheme(
            primary = colors.accent, onPrimary = colors.onAccent,
            background = colors.background, surface = colors.background,
            surfaceContainerLow = colors.sidepane, outline = colors.border,
        )
    }
    CompositionLocalProvider(LocalQuireColors provides colors) {
        MaterialTheme(
            colorScheme = scheme,
            typography = if (desktop) DesktopTypography else Typography(),
            content = content,
        )
    }
}

/** Access to the app's own tokens, beside `MaterialTheme`. */
object QuireTheme {
    val colors: QuireColors
        @Composable @ReadOnlyComposable get() = LocalQuireColors.current
}
