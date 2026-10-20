package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.AttachedPdf
import dev.neurokitchen.quire.domain.model.ImportedPdf
import dev.neurokitchen.quire.domain.model.PdfCheck
import dev.neurokitchen.quire.domain.model.PdfFetch
import dev.neurokitchen.quire.domain.model.PdfState
import dev.neurokitchen.quire.domain.model.PdfVerdict
import dev.neurokitchen.quire.domain.repository.AttachmentRepository
import dev.neurokitchen.quire.ffi.AttachedRecord

/** A reference's PDFs and stored text through the engine (`quire-ffi`). */
class EngineAttachmentRepository(private val session: EngineSession) : AttachmentRepository {
    override suspend fun getPdf(id: String): PdfFetch {
        val fetch = session.library().getPdf(id)
        return PdfFetch(fetch.downloaded, fetch.source, fetch.url, fetch.files, fetch.verdict?.let(::verdict))
    }

    override suspend fun attachPdf(id: String, path: String, strict: Boolean) =
        session.library().attachPdf(id, path, strict).toDomain()

    override suspend fun importPdf(path: String, projectPath: String?): ImportedPdf {
        val imported = session.library().importPdf(path, projectPath)
        return ImportedPdf(imported.added.toDomain(), imported.attached.toDomain())
    }

    override suspend fun removePdfs(id: String) = session.library().removePdfs(id).toInt()

    override suspend fun pdfStates() = session.library().pdfStatuses().associate { status ->
        status.id to (if (status.available) PdfState.Available(status.pages?.toInt()) else PdfState.Missing(status.missingReason.orEmpty()))
    }

    override suspend fun verifyPdfs(ids: List<String>) =
        session.library().verifyPdfs(ids).map { PdfCheck(it.id, it.path, verdict(it.verdict)) }

    override suspend fun ocr(id: String) = session.library().ocr(id)
    override suspend fun browserPage(id: String) = session.library().browserPage(id)
    override suspend fun downloadsFolder() = session.library().downloadsDir()
    override suspend fun fillAbstract(id: String, fromPdf: Boolean) = session.library().fillAbstract(id, fromPdf)
    override suspend fun forgetText(id: String) = session.library().forgetText(id).toInt()

    private fun AttachedRecord.toDomain() = AttachedPdf(reference.toDomain(), new, path, verdict(verdict))

    private fun verdict(text: String) = when (text) {
        "identifier" -> PdfVerdict.Identifier
        "title" -> PdfVerdict.Title
        "different" -> PdfVerdict.Different
        "unlikely" -> PdfVerdict.Unlikely
        else -> PdfVerdict.Unknown
    }
}
