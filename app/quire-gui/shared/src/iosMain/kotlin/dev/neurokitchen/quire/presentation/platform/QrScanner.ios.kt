package dev.neurokitchen.quire.presentation.platform

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import platform.UIKit.UIDevice

/** Not yet on iOS (AVFoundation's metadata output, or VisionKit's DataScannerViewController). */
@Composable
actual fun rememberQrScanner(onScanned: (String) -> Unit): QrScanner = remember {
    object : QrScanner {
        override val canScan = false
        override fun scan() {}
    }
}

actual fun deviceName(): String = UIDevice.currentDevice.name
