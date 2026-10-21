package dev.neurokitchen.quire.presentation.platform

import androidx.compose.runtime.Composable

/**
 * The system bars' icons follow Quire's theme, not the system's: light icons on a dark theme.
 * Android only; elsewhere nothing to do.
 */
@Composable
expect fun SystemBarsAppearance(darkTheme: Boolean)
