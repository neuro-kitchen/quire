package dev.neurokitchen.quire.data.repository

import dev.neurokitchen.quire.domain.model.BlockKind
import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.domain.model.PageRect
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.Paper
import dev.neurokitchen.quire.domain.model.PaperTab
import dev.neurokitchen.quire.domain.model.PaperTabKind
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.SelectedTextRange
import dev.neurokitchen.quire.domain.model.TextBlock
import dev.neurokitchen.quire.domain.model.Word
import dev.neurokitchen.quire.domain.model.WordLocation
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map

/** A paper kept in memory: tests, previews, platforms without the engine bridge. No PDF. */
class SampleReaderRepository(
    initialHighlights: Map<String, List<Highlight>> = emptyMap(),
    initialNotes: Map<String, List<Note>> = emptyMap(),
    private val fakePdfs: Boolean = false,
) : ReaderRepository {
    private val highlights = MutableStateFlow<Map<String, List<Highlight>>>(initialHighlights)
    private val notes = MutableStateFlow<Map<String, List<Note>>>(initialNotes)
    private val readings = mutableMapOf<String, ReadingStatus>()
    private var next = 0

    override suspend fun paper(id: String): Paper {
        val reference = SampleLibraryRepository.REFERENCES.firstOrNull { it.id == id }
            ?: SampleLibraryRepository.EXTENDED_REFERENCES.first { it.id == id }
        val abstract = listOf(
            block(BlockKind.Heading(1), reference.title),
            block(BlockKind.Paragraph, reference.creators.joinToString(", ") { it.displayName }, italic = true),
            block(BlockKind.Heading(3), "Abstract"),
            block(BlockKind.Paragraph, reference.abstract.ifEmpty { "No abstract stored." }),
        )
        val body = listOf(
            block(BlockKind.Heading(2), "1 Introduction"),
            block(BlockKind.Paragraph, PARAGRAPH),
            block(BlockKind.Heading(2), "2 Results"),
            block(BlockKind.Paragraph, "Cells fired in one place of the arena and were silent elsewhere."),
        )
        return Paper(listOf(PaperTab(PaperTabKind.Abstract, abstract), PaperTab(PaperTabKind.Paper, body)), source = "Sample")
    }

    override suspend fun pdfFiles(id: String): List<String> {
        if (!fakePdfs) return emptyList()
        val all = SampleLibraryRepository.REFERENCES + SampleLibraryRepository.EXTENDED_REFERENCES
        val hasPdf = all.firstOrNull { it.id == id }?.hasPdf ?: true
        return if (hasPdf) listOf("sample-paper-$id.pdf") else emptyList()
    }

    override suspend fun figure(url: String) = url
    override suspend fun opened(id: String): ReadingStatus {
        val status = readings[id]?.takeIf { it != ReadingStatus.Unread } ?: ReadingStatus.Reading
        readings[id] = status
        return status
    }
    override suspend fun setReading(id: String, status: ReadingStatus) {
        readings[id] = status
    }

    override fun highlights(id: String): Flow<List<Highlight>> = highlights.map { it[id].orEmpty() }

    override suspend fun addHighlight(id: String, quote: String, prefix: String, comment: String?, color: String) {
        val current = highlights.value[id].orEmpty()
        highlights.value += id to (current + Highlight("h${next++}", quote, prefix, comment, color))
    }

    override suspend fun commentHighlight(highlightId: String, comment: String) {
        highlights.value = highlights.value.mapValues { (_, list) ->
            list.map { if (it.id == highlightId) it.copy(comment = comment) else it }
        }
    }

    override suspend fun removeHighlight(highlightId: String) {
        highlights.value = highlights.value.mapValues { (_, list) ->
            list.filter { it.id != highlightId }
        }
    }

    override fun notes(id: String): Flow<List<Note>> = notes.map { it[id].orEmpty() }

    override suspend fun addNote(id: String, body: String) {
        val current = notes.value[id].orEmpty()
        notes.value += id to (current + Note("n${next++}", body, "2026-10-20"))
    }

    override suspend fun editNote(noteId: String, body: String) {
        notes.value = notes.value.mapValues { (_, list) ->
            list.map { if (it.id == noteId) it.copy(body = body) else it }
        }
    }

    override suspend fun removeNote(noteId: String) {
        notes.value = notes.value.mapValues { (_, list) ->
            list.filter { it.id != noteId }
        }
    }

    private fun block(kind: BlockKind, text: String, italic: Boolean = false) =
        TextBlock(kind, text.split(' ').map { Word(it, italic = italic) })

    companion object {
        const val PARAGRAPH = "Place cells in the hippocampus fire when the animal is in one part of its environment. " +
            "Together they form a map of space that the animal uses to navigate."

        val SAMPLE_HIGHLIGHTS = mapOf(
            "1" to listOf(
                Highlight("h-1-1", "A quantitative description of membrane current", "Title: ", "Seminal model of action potentials", "yellow"),
                Highlight("h-1-2", "conduction and excitation in nerve", "for ", "Voltage-gated ion channels", "green"),
            ),
            "3" to listOf(
                Highlight("h-3-1", "The hippocampus as a spatial map", "Title: ", "Place cells discovery", "yellow"),
                Highlight("h-3-2", "units that fire only when the rat is in a particular place", "preliminary: ", "Cognitive map verification", "blue"),
            ),
            "7" to listOf(
                Highlight("h-7-1", "Attention is all you need", "Title: ", "Transformer architecture introduction", "yellow"),
                Highlight("h-7-2", "dispensing with recurrence and convolutions entirely", "models: ", "Self-attention mechanism", "purple"),
            ),
        )

        val SAMPLE_NOTES = mapOf(
            "1" to listOf(
                Note("n-1-1", "Nobel Prize in Physiology or Medicine 1963 for this work.", "2026-10-18"),
            ),
            "3" to listOf(
                Note("n-3-1", "Key experiment: Unit recordings from dorsal hippocampus in freely moving rodents.", "2026-10-19"),
                Note("n-3-2", "Compare with entorhinal cortex grid cells (Hafting et al. 2005).", "2026-10-20"),
            ),
            "7" to listOf(
                Note("n-7-1", "Multi-Head Attention: Q, K, V scaled dot-product attention.", "2026-10-20"),
            ),
        )
    }
}

/**
 * Rich in-memory fake PDF engine. Renders crisp synthetic multi-page academic articles
 * to RGBA pixel buffers, complete with headers, text columns, figures, and bounding box hit-testing.
 */
class SamplePdfRepository : PdfRepository {
    private val standardPage = PageSize(595f, 842f) // Standard A4 in points (1/72 inch)

    override suspend fun pageSizes(path: String): List<PageSize> = listOf(
        standardPage,
        standardPage,
        standardPage,
    )

    override suspend fun render(path: String, index: Int, width: Int): PageImage {
        val height = (width * (standardPage.height / standardPage.width)).toInt().coerceAtLeast(100)
        val pixels = ByteArray(width * height * 4)

        // 1. Fill crisp paper background (#FCFCFD)
        for (i in 0 until (width * height)) {
            val offset = i * 4
            pixels[offset] = 0xFC.toByte()     // R
            pixels[offset + 1] = 0xFC.toByte() // G
            pixels[offset + 2] = 0xFD.toByte() // B
            pixels[offset + 3] = 0xFF.toByte() // A
        }

        val scale = width.toFloat() / standardPage.width

        fun drawRectPx(left: Int, top: Int, right: Int, bottom: Int, r: Int, g: Int, b: Int, a: Int = 255) {
            val cLeft = left.coerceIn(0, width - 1)
            val cTop = top.coerceIn(0, height - 1)
            val cRight = right.coerceIn(0, width - 1)
            val cBottom = bottom.coerceIn(0, height - 1)
            for (y in cTop..cBottom) {
                val rowOffset = y * width * 4
                for (x in cLeft..cRight) {
                    val idx = rowOffset + x * 4
                    pixels[idx] = r.toByte()
                    pixels[idx + 1] = g.toByte()
                    pixels[idx + 2] = b.toByte()
                    pixels[idx + 3] = a.toByte()
                }
            }
        }

        fun drawPointRect(leftPt: Float, topPt: Float, rightPt: Float, bottomPt: Float, r: Int, g: Int, b: Int) {
            drawRectPx(
                (leftPt * scale).toInt(),
                (topPt * scale).toInt(),
                (rightPt * scale).toInt(),
                (bottomPt * scale).toInt(),
                r, g, b,
            )
        }

        // Header rule
        drawPointRect(50f, 40f, 545f, 41f, 0xCC, 0xD0, 0xD5)

        when (index) {
            0 -> {
                // Page 0: Title, Authors, Abstract box, and Two Columns
                // Running header
                drawPointRect(50f, 32f, 220f, 36f, 0x88, 0x90, 0x9A)

                // Large Title
                drawPointRect(50f, 70f, 500f, 82f, 0x1A, 0x20, 0x2C)
                drawPointRect(50f, 90f, 420f, 102f, 0x1A, 0x20, 0x2C)

                // Author Byline
                drawPointRect(50f, 116f, 340f, 124f, 0x4A, 0x55, 0x68)
                drawPointRect(50f, 130f, 260f, 136f, 0x71, 0x80, 0x96)

                // Abstract background container
                drawPointRect(50f, 155f, 545f, 255f, 0xF1, 0xF5, 0xF9)
                drawPointRect(50f, 155f, 54f, 255f, 0x3B, 0x82, 0xF6) // Blue accent stripe
                // Abstract text lines
                for (line in 0..4) {
                    val y = 170f + line * 16f
                    val right = if (line == 4) 420f else 530f
                    drawPointRect(64f, y, right, y + 8f, 0x47, 0x55, 0x69)
                }

                // Two columns of body text
                // Left column: 50f to 285f
                for (line in 0..25) {
                    val y = 280f + line * 18f
                    val right = if (line % 6 == 5) 230f else 285f
                    drawPointRect(50f, y, right, y + 7f, 0x33, 0x41, 0x55)
                }
                // Right column: 310f to 545f
                for (line in 0..25) {
                    val y = 280f + line * 18f
                    val right = if (line % 7 == 6) 480f else 545f
                    drawPointRect(310f, y, right, y + 7f, 0x33, 0x41, 0x55)
                }
            }
            1 -> {
                // Page 1: Headings, Text columns, and a Figure Diagram box
                drawPointRect(50f, 32f, 180f, 36f, 0x88, 0x90, 0x9A)

                // Section heading
                drawPointRect(50f, 65f, 200f, 76f, 0x1E, 0x29, 0x3B)

                // Column 1 lines
                for (line in 0..10) {
                    val y = 90f + line * 18f
                    drawPointRect(50f, y, 285f, y + 7f, 0x33, 0x41, 0x55)
                }
                // Column 2 lines
                for (line in 0..10) {
                    val y = 90f + line * 18f
                    drawPointRect(310f, y, 545f, y + 7f, 0x33, 0x41, 0x55)
                }

                // Figure Container (framed box)
                drawPointRect(50f, 310f, 545f, 490f, 0xE2, 0xE8, 0xF0)
                drawPointRect(52f, 312f, 543f, 488f, 0xFF, 0xFF, 0xFF)
                // Inside Figure: mock chart bars
                val barColors = listOf(
                    Triple(0x3B, 0x82, 0xF6),
                    Triple(0x10, 0xB9, 0x81),
                    Triple(0xF5, 0x9E, 0x0B),
                    Triple(0x8B, 0x5C, 0xF6),
                )
                barColors.forEachIndexed { idx, (r, g, b) ->
                    val bx = 100f + idx * 100f
                    val bh = 50f + idx * 25f
                    drawPointRect(bx, 450f - bh, bx + 50f, 450f, r, g, b)
                }
                // Caption
                drawPointRect(70f, 502f, 480f, 510f, 0x64, 0x74, 0x8B)

                // Text underneath figure
                for (line in 0..12) {
                    val y = 530f + line * 18f
                    drawPointRect(50f, y, 285f, y + 7f, 0x33, 0x41, 0x55)
                    drawPointRect(310f, y, 545f, y + 7f, 0x33, 0x41, 0x55)
                }
            }
            2 -> {
                // Page 2: References / Bibliography list
                drawPointRect(50f, 32f, 150f, 36f, 0x88, 0x90, 0x9A)
                drawPointRect(50f, 65f, 180f, 76f, 0x1E, 0x29, 0x3B)

                for (line in 0..32) {
                    val y = 95f + line * 20f
                    val col = if (line % 2 == 0) 50f else 310f
                    val right = col + 220f
                    drawPointRect(col, y, col + 25f, y + 7f, 0x25, 0x63, 0xEB) // Ref number/bracket
                    drawPointRect(col + 32f, y, right, y + 7f, 0x47, 0x55, 0x69)
                    drawPointRect(col + 32f, y + 9f, right - 40f, y + 15f, 0x94, 0xA3, 0xB8)
                }
            }
        }

        // Footer rule and page number
        drawPointRect(50f, 805f, 545f, 806f, 0xCC, 0xD0, 0xD5)
        drawPointRect(280f, 814f, 315f, 822f, 0x64, 0x74, 0x8B)

        return PageImage(width, height, pixels)
    }

    override suspend fun locate(path: String, highlights: List<Highlight>): List<QuoteLocation> {
        return highlights.mapIndexed { idx, highlight ->
            val page = idx % 3
            val y = 280f + (idx * 36f).coerceAtMost(300f)
            QuoteLocation(
                highlightId = highlight.id,
                page = page,
                rects = listOf(
                    PageRect(left = 50f, top = y, right = 285f, bottom = y + 14f),
                    PageRect(left = 50f, top = y + 18f, right = 220f, bottom = y + 32f),
                ),
            )
        }
    }

    override suspend fun wordAt(path: String, page: Int, x: Float, y: Float): WordLocation? {
        if (x !in 40f..550f || y !in 50f..800f) return null
        return WordLocation("neuroscience", PageRect(x - 25f, y - 8f, x + 25f, y + 8f))
    }

    override suspend fun nearestGlyph(path: String, page: Int, x: Float, y: Float): Int? {
        if (y < 40f || y > 810f) return null
        val line = ((y - 60f) / 18f).toInt().coerceAtLeast(0)
        val col = ((x - 50f) / 6f).toInt().coerceAtLeast(0)
        return line * 80 + col
    }

    override suspend fun textRange(path: String, page: Int, start: Int, end: Int): SelectedTextRange? {
        val s = minOf(start, end)
        val e = maxOf(start, end)
        if (s == e) return null

        val startLine = s / 80
        val endLine = e / 80
        val rects = mutableListOf<PageRect>()
        for (line in startLine..endLine) {
            val y = 100f + line * 18f
            val left = if (line == startLine) 50f + (s % 80) * 6f else 50f
            val right = if (line == endLine) 50f + (e % 80) * 6f else 545f
            if (right > left) {
                rects.add(PageRect(left.coerceAtLeast(50f), y, right.coerceAtMost(545f), y + 14f))
            }
        }
        return SelectedTextRange(
            text = "Highlighted sample passage from the research paper illustrating neural dynamics.",
            rects = if (rects.isNotEmpty()) rects else listOf(PageRect(50f, 150f, 300f, 164f)),
        )
    }

    override suspend fun pageText(path: String, page: Int): String? = when (page) {
        0 -> "A quantitative analysis of neural representations and synaptic plasticity mechanisms."
        1 -> "Experimental results demonstrating receptive fields, place cell activity, and cortical architectures."
        2 -> "References and bibliography acknowledging foundational work across neurobiology and machine learning."
        else -> null
    }
}
