package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.AttachedPdf
import dev.neurokitchen.quire.domain.model.ImportedPdf
import dev.neurokitchen.quire.domain.model.PdfCheck
import dev.neurokitchen.quire.domain.model.PdfFetch
import dev.neurokitchen.quire.domain.model.PdfState

/** A reference's PDFs and stored text, through the engine; `id` is a reference ID or cite key. Failures throw. */
interface AttachmentRepository {
    /** Finds, downloads and attaches a PDF unless it has one; a blocked site is recorded (then [browserPage]). */
    suspend fun getPdf(id: String): PdfFetch = unsupported()
    /** Attaches the PDF at `path`; [strict] refuses a PDF of another paper. */
    suspend fun attachPdf(id: String, path: String, strict: Boolean = true): AttachedPdf = unsupported()
    /** Adds the work the PDF at `path` describes and attaches it. */
    suspend fun importPdf(path: String, projectPath: String? = null): ImportedPdf = unsupported()
    /** Removes every PDF of a reference (the stored files stay); returns how many. */
    suspend fun removePdfs(id: String): Int = unsupported()
    /** The PDF state of every reference that has a PDF or a failed attempt, by reference ID. */
    suspend fun pdfStates(): Map<String, PdfState> = unsupported()
    suspend fun verifyPdfs(ids: List<String>): List<PdfCheck> = unsupported()
    /** OCR of a scanned paper: `true` if its PDF got a text layer, `false` if only its text was read. */
    suspend fun ocr(id: String): Boolean = unsupported()
    /** The page to open in the browser to get a blocked PDF by hand. */
    suspend fun browserPage(id: String): String? = unsupported()
    /** Where the browser saves downloads. */
    suspend fun downloadsFolder(): String? = unsupported()
    /** Fills a missing abstract (Europe PMC, then with [fromPdf] the PDF's text); the abstract if found. */
    suspend fun fillAbstract(id: String, fromPdf: Boolean = true): String? = unsupported()
    /** Forgets a paper's stored text so the reader converts it again; returns how many texts. */
    suspend fun forgetText(id: String): Int = unsupported()
}
