package dev.neurokitchen.quire.presentation.platform

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

/**
 * The Storage Access Framework's picker (no storage permission needed). Each chosen document is
 * copied into `cache/picked/` under its own name, and that path is what the app gets.
 */
@Composable
actual fun rememberFilePicker(onPicked: (PickKind, List<String>) -> Unit): FilePicker {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val callback by rememberUpdatedState(onPicked)
    var pending by remember { mutableStateOf(PickKind.Pdfs) }
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
        if (uris.isEmpty()) return@rememberLauncherForActivityResult
        val kind = pending
        scope.launch {
            val paths = withContext(Dispatchers.IO) { uris.mapNotNull { copyDocumentToCache(context, it) } }
            if (paths.isNotEmpty()) callback(kind, if (kind == PickKind.Bib) paths.take(1) else paths)
        }
    }
    return remember {
        object : FilePicker {
            override val canPickFolders = false
            override fun pick(kind: PickKind) {
                pending = kind
                when (kind) {
                    PickKind.Pdfs -> launcher.launch(arrayOf("application/pdf"))
                    // .bib has no registered type on most devices: any text, or any file.
                    PickKind.Bib -> launcher.launch(arrayOf("application/x-bibtex", "text/x-bibtex", "text/plain", "application/octet-stream"))
                    PickKind.Folder -> {}
                }
            }
        }
    }
}

/**
 * The document at [uri] copied to the cache, as a file with its display name; `null` if unreadable.
 * (Picked files, and files shared with Quire or opened in it.)
 */
fun copyDocumentToCache(context: Context, uri: Uri): String? = runCatching {
    val name = context.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
        if (cursor.moveToFirst()) cursor.getString(0) else null
    } ?: "picked-${System.currentTimeMillis()}"
    val dir = File(context.cacheDir, "picked").apply { mkdirs() }
    val target = File(dir, name.replace('/', '_'))
    context.contentResolver.openInputStream(uri)?.use { input -> target.outputStream().use { input.copyTo(it) } } ?: return null
    target.path
}.getOrNull()
