package dev.neurokitchen.quire

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.core.content.IntentCompat
import androidx.lifecycle.lifecycleScope
import dev.neurokitchen.quire.presentation.mobile.share.ShareInbox
import dev.neurokitchen.quire.presentation.mobile.share.SharedItem
import dev.neurokitchen.quire.presentation.platform.copyDocumentToCache
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        // A share or "Open with" that started the app (not again after a configuration change).
        if (savedInstanceState == null) receive(intent)
        setContent { App() }
    }

    /** A share while Quire is open (the activity is single-top). */
    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        receive(intent)
    }

    /** Text (a link, a DOI) or PDFs shared with Quire, or a PDF opened with it: to Add. */
    private fun receive(intent: Intent) {
        val uris: List<Uri> = when (intent.action) {
            Intent.ACTION_SEND -> listOfNotNull(IntentCompat.getParcelableExtra(intent, Intent.EXTRA_STREAM, Uri::class.java))
            Intent.ACTION_SEND_MULTIPLE -> IntentCompat.getParcelableArrayListExtra(intent, Intent.EXTRA_STREAM, Uri::class.java).orEmpty()
            Intent.ACTION_VIEW -> listOfNotNull(intent.data)
            else -> return
        }
        if (uris.isNotEmpty()) {
            lifecycleScope.launch {
                val paths = withContext(Dispatchers.IO) { uris.mapNotNull { copyDocumentToCache(this@MainActivity, it) } }
                if (paths.isNotEmpty()) ShareInbox.post(SharedItem.Files(paths))
            }
            return
        }
        intent.getStringExtra(Intent.EXTRA_TEXT)?.takeIf { it.isNotBlank() }?.let { ShareInbox.post(SharedItem.Text(it)) }
    }
}
