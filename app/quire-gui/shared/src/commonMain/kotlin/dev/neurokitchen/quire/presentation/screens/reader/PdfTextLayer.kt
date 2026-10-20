package dev.neurokitchen.quire.presentation.screens.reader

import dev.neurokitchen.quire.domain.model.SelectedTextRange
import dev.neurokitchen.quire.domain.model.WordLocation

/**
 * The open PDF's text layer, for selecting on its pages: positions in points from the page's
 * top-left, glyphs by their index on the page. Answers `null` without a PDF or text there.
 */
interface PdfTextLayer {
    suspend fun glyphAt(page: Int, x: Float, y: Float): Int? = null
    /** The text and line boxes of glyphs `start` until `end`. */
    suspend fun range(page: Int, start: Int, end: Int): SelectedTextRange? = null
    suspend fun wordAt(page: Int, x: Float, y: Float): WordLocation? = null

    /** No text layer (previews, no PDF). */
    object None : PdfTextLayer
}
