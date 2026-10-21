package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.domain.model.PageRect
import dev.neurokitchen.quire.domain.model.PdfMatch
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.domain.model.SelectedTextRange
import dev.neurokitchen.quire.domain.model.WordLocation
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.ffi.pdfFind
import dev.neurokitchen.quire.ffi.pdfNearestGlyph
import dev.neurokitchen.quire.ffi.pdfPageSizes
import dev.neurokitchen.quire.ffi.pdfPageText
import dev.neurokitchen.quire.ffi.pdfSearch
import dev.neurokitchen.quire.ffi.pdfRenderPage
import dev.neurokitchen.quire.ffi.pdfTextRange
import dev.neurokitchen.quire.ffi.pdfWordAt
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.util.LinkedHashMap

/**
 * PDF pages and their text layer through the engine (hayro, pure Rust: no native library to ship).
 * A render superseded by a newer one for the same page fails with "cancelled": skip it.
 * Offloaded to Dispatchers.IO with a bounded LRU memory cache for seamless scrolling.
 */
class EnginePdfRepository : PdfRepository {
    override suspend fun pageSizes(path: String): List<PageSize> = withContext(Dispatchers.IO) {
        pdfPageSizes(path).map { PageSize(it.width, it.height) }
    }

    // Not cached here: the reader keeps the converted images (a cache of raw pixels here held
    // 10-44 MB per page, twice over).
    override suspend fun render(path: String, index: Int, width: Int): PageImage = withContext(Dispatchers.IO) {
        val page = pdfRenderPage(path, index.toUInt(), width.toUInt())
        PageImage(page.width.toInt(), page.height.toInt(), page.pixels)
    }

    override suspend fun locate(path: String, highlights: List<Highlight>): List<QuoteLocation> = withContext(Dispatchers.IO) {
        pdfFind(path, highlights.map { it.quote }).map { match ->
            QuoteLocation(
                highlightId = highlights[match.quote.toInt()].id,
                page = match.page.toInt(),
                rects = match.rects.map { PageRect(it.left, it.top, it.right, it.bottom) },
            )
        }
    }

    override suspend fun wordAt(path: String, page: Int, x: Float, y: Float): WordLocation? = withContext(Dispatchers.IO) {
        val word = pdfWordAt(path, page.toUInt(), x, y) ?: return@withContext null
        WordLocation(word.text, PageRect(word.left, word.top, word.right, word.bottom))
    }

    override suspend fun nearestGlyph(path: String, page: Int, x: Float, y: Float): Int? = withContext(Dispatchers.IO) {
        pdfNearestGlyph(path, page.toUInt(), x, y)?.toInt()
    }

    override suspend fun textRange(path: String, page: Int, start: Int, end: Int): SelectedTextRange = withContext(Dispatchers.IO) {
        val res = pdfTextRange(path, page.toUInt(), start.toUInt(), end.toUInt())
        SelectedTextRange(
            text = res.text,
            rects = res.rects.map { PageRect(it.left, it.top, it.right, it.bottom) },
        )
    }

    override suspend fun pageText(path: String, page: Int): String = withContext(Dispatchers.IO) {
        pdfPageText(path, page.toUInt())
    }

    override suspend fun search(path: String, query: String, limit: Int): List<PdfMatch> = withContext(Dispatchers.IO) {
        pdfSearch(path, query, limit.toUInt()).map { m ->
            PdfMatch(m.page.toInt(), m.rects.map { PageRect(it.left, it.top, it.right, it.bottom) })
        }
    }
}
