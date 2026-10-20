package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.Paper
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.SelectedTextRange
import dev.neurokitchen.quire.domain.model.WordLocation
import kotlinx.coroutines.flow.Flow

/** One paper in the reader, through the engine; `id` is a reference ID. Failures throw. */
interface ReaderRepository {
    /** The paper's tabs; its text is fetched and stored on first opening if needed. */
    suspend fun paper(id: String): Paper

    /** Its PDF files, oldest first. */
    suspend fun pdfFiles(id: String): List<String>

    /** The local file of a figure image. */
    suspend fun figure(url: String): String

    /** Marks it opened (unread → reading). */
    suspend fun opened(id: String): ReadingStatus
    suspend fun setReading(id: String, status: ReadingStatus)

    /** Highlights, again after every change. */
    fun highlights(id: String): Flow<List<Highlight>>
    suspend fun addHighlight(id: String, quote: String, prefix: String, comment: String? = null, color: String = "yellow")
    suspend fun commentHighlight(highlightId: String, comment: String)
    suspend fun removeHighlight(highlightId: String)

    /** Notes, again after every change. */
    fun notes(id: String): Flow<List<Note>>
    suspend fun addNote(id: String, body: String)
    suspend fun editNote(noteId: String, body: String)
    suspend fun removeNote(noteId: String)
}

/** PDF pages (PDFium). Paths are files from [ReaderRepository.pdfFiles]. */
interface PdfRepository {
    suspend fun pageSizes(path: String): List<PageSize>
    /** Page `index` rendered `width` pixels wide. */
    suspend fun render(path: String, index: Int, width: Int): PageImage
    /** Where each highlight's quote is (first match; missing ones left out). */
    suspend fun locate(path: String, highlights: List<Highlight>): List<QuoteLocation>

    suspend fun wordAt(path: String, page: Int, x: Float, y: Float): WordLocation? = null
    suspend fun nearestGlyph(path: String, page: Int, x: Float, y: Float): Int? = null
    suspend fun textRange(path: String, page: Int, start: Int, end: Int): SelectedTextRange? = null
    suspend fun pageText(path: String, page: Int): String? = null
}
