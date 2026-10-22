package dev.neurokitchen.quire.presentation.platform

import android.content.pm.PackageManager
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.platform.LocalContext

/** `Manifest.permission.ACCESS_LOCAL_NETWORK` (API 37). */
private const val ACCESS_LOCAL_NETWORK = "android.permission.ACCESS_LOCAL_NETWORK"

@Composable
actual fun RequestLocalNetworkAccess() {
    if (Build.VERSION.SDK_INT < 37) return
    val context = LocalContext.current
    // Refused: syncing through a hub at home fails with "no connection to the relay"; it can be
    // allowed later in the system settings.
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) {}
    LaunchedEffect(Unit) {
        if (context.checkSelfPermission(ACCESS_LOCAL_NETWORK) != PackageManager.PERMISSION_GRANTED) {
            launcher.launch(ACCESS_LOCAL_NETWORK)
        }
    }
}
