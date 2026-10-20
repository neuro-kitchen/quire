package dev.neurokitchen.quire.data.repository

import dev.neurokitchen.quire.domain.model.BlockKind
import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.domain.model.PageImage
import dev.neurokitchen.quire.domain.model.PageSize
import dev.neurokitchen.quire.domain.model.Paper
import dev.neurokitchen.quire.domain.model.PaperTab
import dev.neurokitchen.quire.domain.model.PaperTabKind
import dev.neurokitchen.quire.domain.model.QuoteLocation
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.model.TextBlock
import dev.neurokitchen.quire.domain.model.Word
import dev.neurokitchen.quire.domain.repository.PdfRepository
import dev.neurokitchen.quire.domain.repository.ReaderRepository
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map

/** A paper kept in memory: tests, previews, platforms without the engine bridge. No PDF. */
class SampleReaderRepository : ReaderRepository {
    private val highlights = MutableStateFlow<Map<String, List<Highlight>>>(emptyMap())
    private val notes = MutableStateFlow<Map<String, List<Note>>>(emptyMap())
    private val readings = mutableMapOf<String, ReadingStatus>()
    private var next = 0

    override suspend fun paper(id: String): Paper {
        val reference = SampleLibraryRepository.REFERENCES.first { it.id == id }
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

    override suspend fun pdfFiles(id: String) = emptyList<String>()
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
    override suspend fun addHighlight(id: String, quote: String, prefix: String, comment: String?) {
        highlights.value += id to highlights.value[id].orEmpty() + Highlight("h${next++}", quote, prefix, comment)
    }
    override suspend fun commentHighlight(highlightId: String, comment: String) {
        highlights.value = highlights.value.mapValues { (_, list) -> list.map { if (it.id == highlightId) it.copy(comment = comment) else it } }
    }
    override suspend fun removeHighlight(highlightId: String) {
        highlights.value = highlights.value.mapValues { (_, list) -> list.filter { it.id != highlightId } }
    }

    override fun notes(id: String): Flow<List<Note>> = notes.map { it[id].orEmpty() }
    override suspend fun addNote(id: String, body: String) {
        notes.value += id to notes.value[id].orEmpty() + Note("n${next++}", body, "2026-10-20")
    }
    override suspend fun editNote(noteId: String, body: String) {
        notes.value = notes.value.mapValues { (_, list) -> list.map { if (it.id == noteId) it.copy(body = body) else it } }
    }
    override suspend fun removeNote(noteId: String) {
        notes.value = notes.value.mapValues { (_, list) -> list.filter { it.id != noteId } }
    }

    private fun block(kind: BlockKind, text: String, italic: Boolean = false) =
        TextBlock(kind, text.split(' ').map { Word(it, italic = italic) })

    companion object {
        const val PARAGRAPH = "Place cells in the hippocampus fire when the animal is in one part of its environment. " +
            "Together they form a map of space that the animal uses to navigate."
    }
}

/** No PDFs: every call fails, as for a reference without one. */
class SamplePdfRepository : PdfRepository {
    override suspend fun pageSizes(path: String): List<PageSize> = error("no PDF in the sample library")
    override suspend fun render(path: String, index: Int, width: Int): PageImage = error("no PDF in the sample library")
    override suspend fun locate(path: String, highlights: List<Highlight>) = emptyList<QuoteLocation>()
}
