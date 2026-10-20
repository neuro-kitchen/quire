package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.PdfState
import dev.neurokitchen.quire.domain.model.PdfVerdict
import dev.neurokitchen.quire.ffi.openLibraryAt
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** A reference's PDFs through the real engine. */
class EngineAttachmentRepositoryTest {
    /** A one-page PDF printing `text` (the same layout as the engine's test fixture). */
    private fun pdf(text: String): ByteArray {
        val stream = "BT /F1 12 Tf 72 720 Td ($text) Tj ET"
        val objects = listOf(
            "<< /Type /Catalog /Pages 2 0 R >>",
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>",
            "<< /Length ${stream.length} >>\nstream\n$stream\nendstream",
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
        )
        val out = StringBuilder("%PDF-1.4\n")
        val offsets = objects.mapIndexed { i, body -> out.length.also { out.append("${i + 1} 0 obj\n$body\nendobj\n") } }
        val xref = out.length
        out.append("xref\n0 ${objects.size + 1}\n0000000000 65535 f \n")
        offsets.forEach { out.append("%010d 00000 n \n".format(it)) }
        out.append("trailer\n<< /Size ${objects.size + 1} /Root 1 0 R >>\nstartxref\n$xref\n%%EOF\n")
        return out.toString().toByteArray()
    }

    @Test
    fun attachesChecksAndRemovesAPdf() = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val engine = openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString())
            val bib = dir.resolve("refs.bib").also {
                Files.writeString(it, "@article{okeefe1971, title = {The hippocampus as a spatial map}, author = {O'Keefe, J.}, year = {1971}, doi = {10.1016/0006-8993(71)90358-1}}")
            }
            engine.importBib(bib.toString(), null)
            val repository = EngineAttachmentRepository(EngineSession(open = { engine }))
            val file = dir.resolve("paper.pdf").also { Files.write(it, pdf("Brain Research doi:10.1016/0006-8993(71)90358-1")) }

            val attached = repository.attachPdf("okeefe1971", file.toString())
            assertEquals(PdfVerdict.Identifier, attached.verdict)
            assertTrue(attached.isNew && attached.reference.hasPdf)
            assertEquals(PdfState.Available(1), repository.pdfStates()[attached.reference.id])
            assertEquals(listOf(PdfVerdict.Identifier), repository.verifyPdfs(listOf("okeefe1971")).map { it.verdict })
            assertTrue(repository.browserPage("okeefe1971").orEmpty().contains("10.1016"))
            assertEquals(1, repository.removePdfs("okeefe1971"))
            assertTrue(repository.pdfStates()[attached.reference.id] == null)
        }
    }
}
