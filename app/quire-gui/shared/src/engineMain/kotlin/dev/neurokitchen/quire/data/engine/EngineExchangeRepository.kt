package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.AutoExport
import dev.neurokitchen.quire.domain.model.AutoExportRun
import dev.neurokitchen.quire.domain.model.CiteUse
import dev.neurokitchen.quire.domain.model.ExportOptions
import dev.neurokitchen.quire.domain.model.ExportProfile
import dev.neurokitchen.quire.domain.model.ExportSelection
import dev.neurokitchen.quire.domain.model.ManuscriptCheck
import dev.neurokitchen.quire.domain.model.PaperAnnotations
import dev.neurokitchen.quire.domain.model.UnknownCiteKey
import dev.neurokitchen.quire.domain.model.ZoteroImport
import dev.neurokitchen.quire.domain.repository.ExchangeRepository
import dev.neurokitchen.quire.ffi.ExportOptionsRecord
import dev.neurokitchen.quire.ffi.exportEncodings as ffiExportEncodings
import dev.neurokitchen.quire.ffi.exportFormats as ffiExportFormats
import dev.neurokitchen.quire.ffi.ExportSelection as FfiExportSelection

/** Imports, exports and manuscript checks through the engine (`quire-ffi`). */
class EngineExchangeRepository(private val session: EngineSession) : ExchangeRepository {
    override suspend fun importBib(path: String, projectPath: String?) = session.library().importBib(path, projectPath).toInt()
    override suspend fun importBibText(text: String, projectPath: String?) = session.library().importBibText(text, projectPath).toInt()

    override suspend fun importZotero(dir: String, projectPath: String?): ZoteroImport {
        val z = session.library().importZotero(dir, projectPath, null)
        return ZoteroImport(z.added.toInt(), z.existing.toInt(), z.projects.toInt(), z.notes.toInt(), z.pdfs.toInt(), z.highlights.toInt(), z.skipped)
    }

    override fun exportFormats() = ffiExportFormats()

    override fun exportEncodings() = ffiExportEncodings()

    override suspend fun exportProfiles() = session.library().exportProfiles().map { ExportProfile(it.name, it.options.toDomain()) }

    override suspend fun export(selection: ExportSelection): Pair<String, Int> {
        val exported = session.library().export(selection.toFfi())
        return exported.text to exported.count.toInt()
    }

    override suspend fun exportTo(selection: ExportSelection, path: String) =
        session.library().exportTo(selection.toFfi(), path).count.toInt()

    override suspend fun keepUpdated(selection: ExportSelection, path: String) =
        session.library().keepUpdated(selection.toFfi(), path).count.toInt()

    override suspend fun autoExports() = session.library().autoExports().map {
        AutoExport(it.path, it.project, it.recursive, it.profile, it.format, it.written)
    }

    override suspend fun stopAutoExport(path: String) = session.library().stopAutoExport(path)

    override suspend fun runAutoExports(force: Boolean) = session.library().runAutoExports(force).map {
        AutoExportRun(it.path, it.outcome, it.count.toInt(), it.error)
    }

    override suspend fun annotationsMarkdown(projectPath: String?, recursive: Boolean) =
        session.library().annotationsMarkdown(projectPath, recursive).map { PaperAnnotations(it.id, it.citeKey, it.markdown) }

    override suspend fun checkManuscript(files: List<String>, projectPath: String?, recursive: Boolean): ManuscriptCheck {
        val c = session.library().checkManuscript(files, projectPath, recursive)
        return ManuscriptCheck(
            files = c.files,
            cited = c.cited.toInt(),
            unknown = c.unknown.map { k ->
                UnknownCiteKey(k.key, k.uses.map { CiteUse(it.file, it.line.toInt(), it.column.toInt()) }, k.suggestions, k.trashed)
            },
            outside = c.outside,
            uncited = c.uncited,
        )
    }

    private fun ExportSelection.toFfi() =
        FfiExportSelection(projectPath, recursive, references, profile, options?.toFfi(), format)

    private fun ExportOptions.toFfi() = ExportOptionsRecord(format, omit, ascii, monthMacros, preprintsAsArticles, encoding)

    private fun ExportOptionsRecord.toDomain() = ExportOptions(
        format = format ?: "biblatex",
        omit = omit,
        ascii = ascii,
        monthMacros = monthMacros,
        preprintsAsArticles = preprintsAsArticles,
        encoding = encoding ?: "utf-8",
    )
}
