package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.AutoExport
import dev.neurokitchen.quire.domain.model.AutoExportRun
import dev.neurokitchen.quire.domain.model.ExportSelection
import dev.neurokitchen.quire.domain.model.ManuscriptCheck
import dev.neurokitchen.quire.domain.model.PaperAnnotations
import dev.neurokitchen.quire.domain.model.ZoteroImport

/** References in and out of the library, through the engine. Failures throw. */
interface ExchangeRepository {
    /** Imports a `.bib` file (into the project at [projectPath] if given); returns how many were new. */
    suspend fun importBib(path: String, projectPath: String? = null): Int = unsupported()
    /** Imports pasted `.bib` text; returns how many references were new. */
    suspend fun importBibText(text: String, projectPath: String? = null): Int = unsupported()
    /** Imports the Zotero library in the data folder [dir]. */
    suspend fun importZotero(dir: String, projectPath: String? = null): ZoteroImport = unsupported()
    /** The export format names. */
    fun exportFormats(): List<String> = unsupported()
    /** The selected references as text, and how many. */
    suspend fun export(selection: ExportSelection): Pair<String, Int> = unsupported()
    /** Writes the selection to [path] once; returns how many references. */
    suspend fun exportTo(selection: ExportSelection, path: String): Int = unsupported()
    /** Writes the selection to [path] now and after every change; returns how many references. */
    suspend fun keepUpdated(selection: ExportSelection, path: String): Int = unsupported()
    suspend fun autoExports(): List<AutoExport> = unsupported()
    /** Stops rewriting [path] (the file stays); `false` if it was not an automatic export. */
    suspend fun stopAutoExport(path: String): Boolean = unsupported()
    suspend fun runAutoExports(force: Boolean = false): List<AutoExportRun> = unsupported()
    /** Each paper with highlights or notes in the project (or the library), as Markdown. */
    suspend fun annotationsMarkdown(projectPath: String? = null, recursive: Boolean = true): List<PaperAnnotations> = unsupported()
    /** Checks the cite keys of [files] (`.typ`, `.tex`, `.md`; folders are searched). */
    suspend fun checkManuscript(files: List<String>, projectPath: String? = null, recursive: Boolean = true): ManuscriptCheck = unsupported()
}
