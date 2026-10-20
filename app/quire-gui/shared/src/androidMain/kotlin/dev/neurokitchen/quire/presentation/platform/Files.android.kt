package dev.neurokitchen.quire.presentation.platform

/** Not yet: Android needs the system picker through an activity result, with the engine bridge. */
actual fun pickFile(title: String, extensions: List<String>): String? = null

actual fun pickFolder(title: String): String? = null

actual fun pickSaveFile(title: String, name: String): String? = null
