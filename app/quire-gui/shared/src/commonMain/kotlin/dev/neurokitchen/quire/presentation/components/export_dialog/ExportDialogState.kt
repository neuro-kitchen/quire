package dev.neurokitchen.quire.presentation.components.export_dialog

import androidx.compose.runtime.Immutable
import dev.neurokitchen.quire.domain.model.ExportOptions
import dev.neurokitchen.quire.domain.model.ExportProfile
import dev.neurokitchen.quire.presentation.components.notice.Notice

/** What to export: the [references] (IDs) if any, else the project at [projectPath] or the library. */
data class ExportTarget(
    val projectPath: String? = null,
    val recursive: Boolean = true,
    val references: List<String> = emptyList(),
)

/** The encoding choices; [ENCODING_ASCII] is UTF-8 with non-ASCII as LaTeX commands (.bib). */
const val ENCODING_UTF8 = "utf-8"
const val ENCODING_UTF8_BOM = "utf-8-bom"
const val ENCODING_WINDOWS_1252 = "windows-1252"
const val ENCODING_ASCII = "ascii"

/** A field that can be left out, by its name in .bib (and Hayagriva) and in CSL-JSON (`null`: none). */
enum class OmitField(val bib: String, val csl: String?) {
    Abstract("abstract", "abstract"),
    Note("note", "note"),
    Keywords("keywords", "keyword"),
    Url("url", "URL"),
    File("file", null),
    ;

    fun nameIn(format: String): String? = if (format == "csl-json") csl else bib
}

@Immutable
data class ExportDialogState(
    val target: ExportTarget = ExportTarget(),
    val formats: List<String> = emptyList(),
    /** The encodings the format can be written in. */
    val encodings: List<String> = emptyList(),
    val profiles: List<ExportProfile> = emptyList(),
    /** The profile chosen, sent by name while its options are unchanged; `null`: the options here. */
    val profile: String? = null,
    /** Format, months and preprints; the encoding and left-out fields are [encoding] and [omitted]. */
    val options: ExportOptions = ExportOptions(),
    val encoding: String = ENCODING_UTF8,
    val omitted: Set<OmitField> = emptySet(),
    /** The export as it will be written, and how many references. */
    val preview: String? = null,
    val count: Int = 0,
    val isWorking: Boolean = false,
    /** Why the export cannot be made as chosen (from the engine). */
    val error: String? = null,
    /** What saving did; the dialog closes and its host shows it. */
    val done: Notice.Done? = null,
) {
    val isBib: Boolean get() = options.format == "biblatex" || options.format == "bibtex"

    /** A selection is written once: what to add to it later is not known. */
    val canKeepUpdated: Boolean get() = target.references.isEmpty()
}
