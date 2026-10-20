package dev.neurokitchen.quire.presentation.platform

/**
 * Asks the user for a file (the system's dialog), of one of `extensions` (`pdf`, `bib`; empty: any).
 * Returns its path, or `null` if cancelled or not available on this platform yet.
 */
expect fun pickFile(title: String, extensions: List<String> = emptyList()): String?

/** Asks the user for a folder; its path, or `null` (cancelled, or not available here yet). */
expect fun pickFolder(title: String): String?

/** Asks the user where to save a file, suggesting [name]; its path, or `null`. */
expect fun pickSaveFile(title: String, name: String): String?
