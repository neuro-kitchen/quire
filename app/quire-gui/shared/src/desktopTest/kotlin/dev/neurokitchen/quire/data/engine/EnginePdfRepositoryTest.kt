package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.tinyPdf
import kotlinx.coroutines.runBlocking
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

/** PDF pages and text layer through the pure-Rust hayro bridge. */
class EnginePdfRepositoryTest {
    @Test
    fun rendersPagesAndLocatesHighlights() = runBlocking {
        val path = Files.createTempFile("paper", ".pdf").also { Files.write(it, tinyPdf(listOf("Place cells fire in one place"))) }.toString()
        val pdf = EnginePdfRepository()
        assertEquals(1, pdf.pageSizes(path).size)
        val page = pdf.render(path, 0, 612)
        assertEquals(612 to 792, page.width to page.height)
        val found = pdf.locate(path, listOf(Highlight("h1", "fire in one"), Highlight("h2", "absent")))
        assertEquals(listOf("h1"), found.map { it.highlightId })
        assertTrue(found[0].rects.isNotEmpty())

        // Text layer tests
        val word = pdf.wordAt(path, 0, 75f, 70f)
        assertNotNull(word)
        assertEquals("Place", word.text)

        val glyph = pdf.nearestGlyph(path, 0, 72f, 70f)
        assertEquals(0, glyph)

        val range = pdf.textRange(path, 0, 0, 5)
        assertNotNull(range)
        assertEquals("Place", range.text)
        assertTrue(range.rects.isNotEmpty())

        val text = pdf.pageText(path, 0)
        assertTrue(text.contains("Place cells fire"))

        // Find: both places, in reading order, on page 0.
        val matches = pdf.search(path, "place")
        assertEquals(listOf(0, 0), matches.map { it.page })
        assertTrue(matches[0].rects[0].left < matches[1].rects[0].left)
    }
}
