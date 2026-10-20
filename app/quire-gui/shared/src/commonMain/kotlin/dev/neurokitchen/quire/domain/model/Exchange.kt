package dev.neurokitchen.quire.domain.model

/** What a Zotero import brought in; [skipped] items not imported, with why. */
data class ZoteroImport(
    val added: Int,
    val existing: Int,
    val projects: Int,
    val notes: Int,
    val pdfs: Int,
    val highlights: Int,
    val skipped: List<String>,
)

/**
 * What to export: the project at [projectPath] (with subprojects if [recursive]) or the library,
 * with a config [profile] and/or a [format] (`biblatex`, `bibtex`, `csl-json`, `hayagriva`).
 */
data class ExportSelection(
    val projectPath: String? = null,
    val recursive: Boolean = true,
    val profile: String? = null,
    val format: String? = null,
)

/** A file rewritten after every change to the library; [written] `YYYY-MM-DD`. */
data class AutoExport(
    val path: String,
    val projectPath: String?,
    val recursive: Boolean,
    val profile: String?,
    val format: String?,
    val written: String?,
)

/** One automatic export's run: [outcome] `written` (with [count]), `unchanged` or `failed` (with [error]). */
data class AutoExportRun(val path: String, val outcome: String, val count: Int, val error: String?)

/** A paper's highlights and notes as a Markdown file's text. */
data class PaperAnnotations(val id: String, val citeKey: String, val markdown: String)

/** Where a manuscript cites a key (line and column from 1). */
data class CiteUse(val file: String, val line: Int, val column: Int)

data class UnknownCiteKey(val key: String, val uses: List<CiteUse>, val suggestions: List<String>, val trashed: Boolean)

/** A manuscript checked against the library (or a project); [outside] and [uncited] are cite keys. */
data class ManuscriptCheck(
    val files: List<String>,
    val cited: Int,
    val unknown: List<UnknownCiteKey>,
    val outside: List<String>,
    val uncited: List<String>,
)
