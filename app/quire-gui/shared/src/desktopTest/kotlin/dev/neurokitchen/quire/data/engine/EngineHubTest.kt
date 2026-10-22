package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.PdfSource
import dev.neurokitchen.quire.ffi.openLibraryAt
import dev.neurokitchen.quire.tinyPdf
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/** The hub and lazy-PDF calls through the real engine that need no network (`.knowledge/sync-hub.md`). */
class EngineHubTest {
    @Test
    fun recoveryKitKeepOfflineOpeningAndEviction(): Unit = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val session = EngineSession(open = { openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString()) })
            val sync = EngineSyncRepository(session)
            val attachments = EngineAttachmentRepository(session)
            val bib = dir.resolve("refs.bib").also {
                Files.writeString(it, "@article{okeefe1971, title = {The hippocampus as a spatial map}, author = {O'Keefe, J.}, year = {1971}}")
            }
            EngineExchangeRepository(session).importBib(bib.toString())
            val pdf = dir.resolve("paper.pdf").also { Files.write(it, tinyPdf(listOf("The hippocampus as a spatial map"))) }
            attachments.attachPdf("okeefe1971", pdf.toString(), strict = false)

            assertTrue(session.library().exportUserKey() in sync.recoveryKit())
            assertTrue(attachments.keepOffline("okeefe1971"))
            val opened = attachments.openPdf("okeefe1971")
            assertEquals(PdfSource.Here, opened.source)
            assertTrue(Files.isRegularFile(java.nio.file.Path.of(opened.path)))
            assertEquals(0, attachments.evictPdfs().removed, "no limit, no hub copy")
            assertFailsWith<Exception> { sync.syncWithHub() }
        }
    }
}
