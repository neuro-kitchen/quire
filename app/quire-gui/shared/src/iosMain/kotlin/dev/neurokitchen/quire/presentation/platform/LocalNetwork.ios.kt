package dev.neurokitchen.quire.presentation.platform

import androidx.compose.runtime.Composable

/** iOS asks by itself on the first local connection (NSLocalNetworkUsageDescription in Info.plist). */
@Composable
actual fun RequestLocalNetworkAccess() = Unit
