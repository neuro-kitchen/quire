package dev.neurokitchen.quire.presentation.platform

import androidx.compose.runtime.Composable

/**
 * Asks, once, for access to the home network where the platform requires it: Android 17+ blocks an
 * app's connections to local devices (a self-hosted hub and relay) without `ACCESS_LOCAL_NETWORK`.
 * Elsewhere it does nothing.
 */
@Composable
expect fun RequestLocalNetworkAccess()
