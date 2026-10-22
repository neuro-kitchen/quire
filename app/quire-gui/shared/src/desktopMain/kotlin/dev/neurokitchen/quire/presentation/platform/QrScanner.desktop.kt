package dev.neurokitchen.quire.presentation.platform

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember

@Composable
actual fun rememberQrScanner(onScanned: (String) -> Unit): QrScanner = remember {
    object : QrScanner {
        override val canScan = false
        override fun scan() = Unit
    }
}

actual fun deviceName(): String =
    runCatching { java.net.InetAddress.getLocalHost().hostName }.getOrNull()?.takeIf { it.isNotBlank() }
        ?: System.getenv("HOSTNAME")
        ?: "Computer"
