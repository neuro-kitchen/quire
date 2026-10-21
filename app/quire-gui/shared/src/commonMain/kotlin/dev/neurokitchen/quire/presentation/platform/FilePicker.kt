package dev.neurokitchen.quire.presentation.platform

import androidx.compose.runtime.Composable

/** What can be picked: PDF files (several), a `.bib` file, a folder (Zotero's data folder). */
enum class PickKind { Pdfs, Bib, Folder }

/**
 * The system's file picker, from a composable: Android's is asynchronous (an activity result), so
 * [pick] returns at once and the paths arrive in `rememberFilePicker`'s callback. On Android the
 * files are copied into the app's cache first (the engine reads paths, the picker gives URIs).
 */
interface FilePicker {
    fun pick(kind: PickKind)

    /** Whether [PickKind.Folder] can be picked here (not on phones). */
    val canPickFolders: Boolean
}

@Composable
expect fun rememberFilePicker(onPicked: (PickKind, List<String>) -> Unit): FilePicker
