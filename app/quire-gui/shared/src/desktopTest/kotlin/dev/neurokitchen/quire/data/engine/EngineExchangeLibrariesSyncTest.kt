package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.ExportSelection
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.ffi.openLibraryAt
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Imports and exports, libraries, sync identity and sharing through the real engine (no network). */
class EngineExchangeLibrariesSyncTest {
    private val bib = "@article{okeefe1971, title = {The hippocampus as a spatial map}, author = {O'Keefe, J.}, year = {1971}}"

    @Test
    fun importsExportsAndChecksAManuscript() = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val session = EngineSession(open = { openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString()) })
            val exchange = EngineExchangeRepository(session)
            val file = dir.resolve("refs.bib").also { Files.writeString(it, bib) }
            assertEquals(1, exchange.importBib(file.toString()))
            assertTrue("hayagriva" in exchange.exportFormats())
            val (text, count) = exchange.export(ExportSelection(format = "biblatex"))
            assertTrue(count == 1 && "okeefe1971" in text)
            val out = dir.resolve("refs.json")
            assertEquals(1, exchange.keepUpdated(ExportSelection(format = "csl-json"), out.toString()))
            assertEquals(listOf("csl-json"), exchange.autoExports().map { it.format })
            assertTrue(exchange.stopAutoExport(out.toString()))

            val draft = dir.resolve("draft.md").also { Files.writeString(it, "See [@okeefe1971] and [@nobody2020].\n") }
            val check = exchange.checkManuscript(listOf(draft.toString()))
            assertEquals(listOf("nobody2020"), check.unknown.map { it.key })
        }
    }

    @Test
    fun registersOpensAndBacksUpLibraries() = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val config = dir.resolve("config.toml").toString()
            val session = EngineSession(open = { openLibraryAt(dir.resolve("first").toString(), config) })
            val libraries = EngineLibrariesRepository(session, config)
            val library = EngineLibraryRepository(session)
            EngineExchangeRepository(session).importBib(dir.resolve("refs.bib").also { Files.writeString(it, bib) }.toString())
            libraries.register("second", dir.resolve("second").toString())
            assertTrue(libraries.libraries().any { it.name == "second" })

            val copy = libraries.copyTo("okeefe1971", "second")
            assertTrue(copy.isNew)
            val backup = libraries.backup(dir.resolve("first.tar.gz").toString())
            assertTrue(backup.files >= 1 && backup.bytes > 0)

            // Open flows follow the switch: the second library has the copy only.
            libraries.open("second")
            assertEquals(listOf("okeefe1971"), library.references(LibraryQuery()).first().map { it.citeKey })
            assertTrue(library.describe().path.endsWith("second"))
        }
    }

    @Test
    fun hasAnIdentityAndSharesProjects() = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val session = EngineSession(open = { openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString()) })
            val sync = EngineSyncRepository(session)
            assertTrue(sync.deviceId().isNotBlank())
            val me = sync.userId()
            assertEquals(64, me.length)
            // The key itself never reaches the app's screens (devices link by QR code); the engine still has it.
            assertEquals(me, session.library().importUserKey(session.library().exportUserKey()))
            EngineLibraryRepository(session).createProject("Lab")
            val colleague = "b".repeat(64)
            assertEquals("editor", sync.shareProject("lab", colleague, "Ana", "editor").role)
            assertTrue(sync.projectMembers("lab").any { it.userId == colleague })
            sync.unshareProject("lab", colleague)
            assertTrue(sync.projectMembers("lab").none { it.userId == colleague })
        }
    }
}
