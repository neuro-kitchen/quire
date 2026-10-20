package dev.neurokitchen.quire.presentation.platform

/** Not yet: iOS needs a document picker, with the engine bridge. */
actual fun pickFile(title: String, extensions: List<String>): String? = null

actual fun pickFolder(title: String): String? = null

actual fun pickSaveFile(title: String, name: String): String? = null
