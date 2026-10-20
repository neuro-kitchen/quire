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

/**
 * PDF pages and their text layer through the engine (hayro, pure Rust: no native library to ship).
 * A render superseded by a newer one for the same page fails with "cancelled": skip it.
 */
class EnginePdfRepository : PdfRepository {
    override suspend fun pageSizes(path: String): List<PageSize> {
        return pdfPageSizes(path).map { PageSize(it.width, it.height) }
    }

    override suspend fun render(path: String, index: Int, width: Int): PageImage {
        val page = pdfRenderPage(path, index.toUInt(), width.toUInt())
        return PageImage(page.width.toInt(), page.height.toInt(), page.pixels)
    }

    override suspend fun locate(path: String, highlights: List<Highlight>): List<QuoteLocation> {
        return pdfFind(path, highlights.map { it.quote }).map { match ->
            QuoteLocation(
                highlightId = highlights[match.quote.toInt()].id,
                page = match.page.toInt(),
                rects = match.rects.map { PageRect(it.left, it.top, it.right, it.bottom) },
            )
        }
    }

    override suspend fun wordAt(path: String, page: Int, x: Float, y: Float): WordLocation? {
        val word = pdfWordAt(path, page.toUInt(), x, y) ?: return null
        return WordLocation(word.text, PageRect(word.left, word.top, word.right, word.bottom))
    }

    override suspend fun nearestGlyph(path: String, page: Int, x: Float, y: Float): Int? {
        return pdfNearestGlyph(path, page.toUInt(), x, y)?.toInt()
    }

    override suspend fun textRange(path: String, page: Int, start: Int, end: Int): SelectedTextRange {
        val res = pdfTextRange(path, page.toUInt(), start.toUInt(), end.toUInt())
        return SelectedTextRange(
            text = res.text,
            rects = res.rects.map { PageRect(it.left, it.top, it.right, it.bottom) },
        )
    }

    override suspend fun pageText(path: String, page: Int): String {
        return pdfPageText(path, page.toUInt())
    }

    override suspend fun search(path: String, query: String, limit: Int): List<PdfMatch> =
        pdfSearch(path, query, limit.toUInt()).map { m -> PdfMatch(m.page.toInt(), m.rects.map { PageRect(it.left, it.top, it.right, it.bottom) }) }
}
