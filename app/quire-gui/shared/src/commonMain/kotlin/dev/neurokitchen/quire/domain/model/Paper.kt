package dev.neurokitchen.quire.domain.model

/** A paper as the reader shows it: the abstract tab, then its text tabs (if its text was found). */
data class Paper(
    val tabs: List<PaperTab>,
    /** Where the text came from (`arXiv HTML`, `Europe PMC`, `PDF`); `null` without text. */
    val source: String? = null,
    /** Why there is no text, when there is none. */
    val textError: String? = null,
)

data class PaperTab(val kind: PaperTabKind, val blocks: List<TextBlock>, val count: Int? = null)

enum class PaperTabKind { Abstract, Paper, Front, Figures, Tables, Equations, References }

data class TextBlock(val kind: BlockKind, val words: List<Word>) {
    val text: String get() = words.joinToString(" ") { it.text }
}

sealed interface BlockKind {
    data class Heading(val level: Int) : BlockKind
    data object Paragraph : BlockKind
    data object ListItem : BlockKind
    data object TableRow : BlockKind
    data object Equation : BlockKind
    /** A figure image; `url` is passed to `ReaderRepository.figure` for a local file. */
    data class Image(val url: String) : BlockKind
}

data class Word(val text: String, val bold: Boolean = false, val italic: Boolean = false, val mention: Boolean = false)

data class Highlight(
    val id: String,
    val quote: String,
    /** Text just before the quote, telling repeated quotes apart. */
    val prefix: String = "",
    val comment: String? = null,
    val color: String = "yellow",
)

data class Note(val id: String, val body: String, val updated: String)

/** A PDF page's size in points (1/72 inch). */
data class PageSize(val width: Float, val height: Float)

/** A rendered page: RGBA pixels, row by row. */
class PageImage(val width: Int, val height: Int, val pixels: ByteArray)

/** A box on a page, in points from its top-left corner. */
data class PageRect(val left: Float, val top: Float, val right: Float, val bottom: Float)

/** Where a highlight's quote is in a PDF. */
data class QuoteLocation(val highlightId: String, val page: Int, val rects: List<PageRect>)

/** A single word hit-tested on a PDF page. */
data class WordLocation(val text: String, val rect: PageRect)

/** Selected text range and union bounding rectangles. */
data class SelectedTextRange(val text: String, val rects: List<PageRect>)
