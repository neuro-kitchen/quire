package dev.neurokitchen.quire.presentation.components.cite

import androidx.compose.runtime.Immutable

/** The ways a reference can be copied to cite it; [engine]: the export format (none: the key). */
enum class CiteFormat(val engine: String?) { Key(null), Biblatex("biblatex"), Bibtex("bibtex"), CslJson("csl-json") }

/**
 * Copying citations: the format being written ([working]), the text to put on the clipboard once
 * written ([copied], taken by the screen), or why it could not be written.
 */
@Immutable
data class CiteState(
    val working: CiteFormat? = null,
    val copied: String? = null,
    val error: String? = null,
)
