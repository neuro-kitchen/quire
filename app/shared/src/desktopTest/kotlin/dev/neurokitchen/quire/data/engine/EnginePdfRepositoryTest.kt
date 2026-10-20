package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.tinyPdf
import kotlinx.coroutines.runBlocking
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** PDF pages through the bridge and the PDFium Gradle downloaded (`quire.pdfium.dir`). */
class EnginePdfRepositoryTest {
    @Test
    fun rendersPagesAndLocatesHighlights() = runBlocking {
        val path = Files.createTempFile("paper", ".pdf").also { Files.write(it, tinyPdf(listOf("Place cells fire in one place"))) }.toString()
        val pdf = EnginePdfRepository { System.getProperty("quire.pdfium.dir") }
        assertEquals(1, pdf.pageSizes(path).size)
        val page = pdf.render(path, 0, 612)
        assertEquals(612 to 792, page.width to page.height)
        val found = pdf.locate(path, listOf(Highlight("h1", "fire in one"), Highlight("h2", "absent")))
        assertEquals(listOf("h1"), found.map { it.highlightId })
        assertTrue(found[0].rects.isNotEmpty())
    }
}
