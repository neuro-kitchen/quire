package dev.neurokitchen.quire.presentation.platform

import android.os.Build
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.platform.LocalContext
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning

@Composable
actual fun rememberQrScanner(onScanned: (String) -> Unit): QrScanner {
    val context = LocalContext.current
    val latest by rememberUpdatedState(onScanned)
    return remember(context) {
        object : QrScanner {
            override val canScan = true

            override fun scan() {
                val options = GmsBarcodeScannerOptions.Builder().setBarcodeFormats(Barcode.FORMAT_QR_CODE).enableAutoZoom().build()
                // Cancelled or failed (no Play services, module not downloaded yet): nothing happens;
                // the code can still be pasted.
                GmsBarcodeScanning.getClient(context, options).startScan().addOnSuccessListener { barcode ->
                    barcode.rawValue?.let { latest(it) }
                }
            }
        }
    }
}

actual fun deviceName(): String = listOf(Build.MANUFACTURER, Build.MODEL).filter { it.isNotBlank() }.distinct().joinToString(" ").ifBlank { "Android device" }
