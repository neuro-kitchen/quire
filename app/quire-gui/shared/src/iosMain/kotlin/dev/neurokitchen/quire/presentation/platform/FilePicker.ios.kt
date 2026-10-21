package dev.neurokitchen.quire.presentation.platform

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember

/** Not yet on iOS (UIDocumentPickerViewController, with the engine bridge). */
@Composable
actual fun rememberFilePicker(onPicked: (PickKind, List<String>) -> Unit): FilePicker = remember {
    object : FilePicker {
        override val canPickFolders = false
        override fun pick(kind: PickKind) {}
    }
}
