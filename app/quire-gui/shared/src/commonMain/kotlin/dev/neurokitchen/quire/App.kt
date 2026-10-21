package dev.neurokitchen.quire

import androidx.compose.runtime.Composable
import dev.neurokitchen.quire.presentation.mobile.MobileApp

/** The phone and tablet app; Android and iOS show it after `initKoin` (the desktop uses `DesktopApp`). */
@Composable
fun App() = MobileApp()
