package dev.neurokitchen.quire.presentation.platform

import java.awt.FileDialog
import java.awt.Frame
import java.io.File
import javax.swing.JFileChooser

/** AWT's native dialog (modal; called from a click, on the UI thread). */
actual fun pickFile(title: String, extensions: List<String>): String? {
    val dialog = FileDialog(null as Frame?, title, FileDialog.LOAD)
    if (extensions.isNotEmpty()) {
        dialog.setFilenameFilter { _, name -> extensions.any { name.endsWith(".$it", ignoreCase = true) } }
    }
    dialog.isVisible = true
    val name = dialog.file ?: return null
    return File(dialog.directory, name).path
}

/** Swing's chooser: AWT's native dialog picks files only (outside macOS). */
actual fun pickFolder(title: String): String? {
    val chooser = JFileChooser().apply {
        dialogTitle = title
        fileSelectionMode = JFileChooser.DIRECTORIES_ONLY
    }
    return if (chooser.showOpenDialog(null) == JFileChooser.APPROVE_OPTION) chooser.selectedFile.path else null
}

actual fun pickSaveFile(title: String, name: String): String? {
    val dialog = FileDialog(null as Frame?, title, FileDialog.SAVE)
    dialog.file = name
    dialog.isVisible = true
    val chosen = dialog.file ?: return null
    return File(dialog.directory, chosen).path
}
