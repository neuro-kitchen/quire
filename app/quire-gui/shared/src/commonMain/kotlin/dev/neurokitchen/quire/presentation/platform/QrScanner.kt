package dev.neurokitchen.quire.presentation.platform

import androidx.compose.runtime.Composable

/**
 * Scanning a QR code (a device's link code): on Android Google's scanner (Play services; the app
 * needs no camera permission, only the text comes back); none on desktop, where codes are pasted.
 */
interface QrScanner {
    val canScan: Boolean
    /** Starts a scan; a code read arrives in `rememberQrScanner`'s callback. */
    fun scan()
}

@Composable
expect fun rememberQrScanner(onScanned: (String) -> Unit): QrScanner

/** This device's name, shown on the other device when linking (the model, the host name). */
expect fun deviceName(): String
