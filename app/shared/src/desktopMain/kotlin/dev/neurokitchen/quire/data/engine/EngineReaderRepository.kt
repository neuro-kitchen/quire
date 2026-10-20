package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.Highlight
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.domain.model.ReadingStatus
import dev.neurokitchen.quire.domain.repository.ReaderRepository

/** A paper's text, highlights, notes and reading status through the engine. */
class EngineReaderRepository(private val session: EngineSession) : ReaderRepository {
    override suspend fun paper(id: String) = session.library().paper(id).toDomain()
    override suspend fun pdfFiles(id: String) = session.library().pdfFiles(id)
    override suspend fun figure(url: String) = session.library().figure(url)
    override suspend fun opened(id: String) = readingStatus(session.library().opened(id))
    override suspend fun setReading(id: String, status: ReadingStatus) =
        session.library().setReading(id, status.name.lowercase())

    override fun highlights(id: String) = session.reloading { library ->
        library.highlights(id).map { Highlight(it.id, it.quote, it.prefix, it.comment) }
    }

    override suspend fun addHighlight(id: String, quote: String, prefix: String, comment: String?) {
        session.library().addHighlight(id, quote, prefix, comment)
    }

    override suspend fun commentHighlight(highlightId: String, comment: String) {
        session.library().commentHighlight(highlightId, comment)
    }

    override suspend fun removeHighlight(highlightId: String) = session.library().removeHighlight(highlightId)

    override fun notes(id: String) = session.reloading { library ->
        library.notes(id).map { Note(it.id, it.body, it.updated) }
    }

    override suspend fun addNote(id: String, body: String) {
        session.library().addNote(id, body)
    }

    override suspend fun editNote(noteId: String, body: String) {
        session.library().editNote(noteId, body)
    }

    override suspend fun removeNote(noteId: String) = session.library().removeNote(noteId)
}
