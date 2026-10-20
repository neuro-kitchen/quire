package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.domain.model.PageRect
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.ffi.pdfFind
import dev.neurokitchen.quire.ffi.pdfInit
import dev.neurokitchen.quire.ffi.pdfPageSizes
import dev.neurokitchen.quire.ffi.pdfRenderPage

/**
 * PDF pages through PDFium. The library is looked for in `pdfiumDir`; the desktop app passes the
 * folder of its bundled copy (Compose app resources), tests the one Gradle downloaded.
 */
class EnginePdfRepository(private val pdfiumDir: () -> String?) : PdfRepository {
    private val ready by lazy { pdfInit(pdfiumDir()) }

    override suspend fun pageSizes(path: String): List<PageSize> {
        ready
        return pdfPageSizes(path).map { PageSize(it.width, it.height) }
    }

    override suspend fun render(path: String, index: Int, width: Int): PageImage {
        ready
        val page = pdfRenderPage(path, index.toUInt(), width.toUInt())
        return PageImage(page.width.toInt(), page.height.toInt(), page.pixels)
    }

    override suspend fun locate(path: String, highlights: List<Highlight>): List<QuoteLocation> {
        ready
        return pdfFind(path, highlights.map { it.quote }).map { match ->
            QuoteLocation(
                highlightId = highlights[match.quote.toInt()].id,
                page = match.page.toInt(),
                rects = match.rects.map { PageRect(it.left, it.top, it.right, it.bottom) },
            )
        }
    }

    companion object {
        /** Where the desktop app finds PDFium: its resources folder, else `quire.pdfium.dir`. */
        fun bundledDir(): String? =
            System.getProperty("compose.application.resources.dir") ?: System.getProperty("quire.pdfium.dir")
    }
}
