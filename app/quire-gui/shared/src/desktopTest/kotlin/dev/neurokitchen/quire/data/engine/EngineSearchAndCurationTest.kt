package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.Creator
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.ffi.openLibraryAt
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Search, saved searches, alerts, duplicates and checks through the real engine (no network). */
class EngineSearchAndCurationTest {
    @Test
    fun searchesAndCurates() = runBlocking {
        withTimeout(30_000) {
            val dir = Files.createTempDirectory("quire-test")
            val engine = openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString())
            val session = EngineSession(open = { engine })
            val library = EngineLibraryRepository(session)
            val search = EngineSearchRepository(session)
            val curation = EngineCurationRepository(session)
            val typed = Reference(
                id = "", citeKey = "", kind = "article-journal", title = "Place cells, grid cells, and the brain's spatial system",
                creators = listOf(Creator("Moser", "Moser, Edvard")), year = 2008,
            )
            library.addReference(typed)
            library.addReference(typed)

            assertEquals(2, search.search("grid cells").size)
            search.saveSearch("grid", "grid cells")
            assertEquals(listOf("grid"), search.savedSearches().map { it.name })
            assertEquals(2, search.runSaved("grid").size)
            search.addAlert("moser", "pubmed", "Moser E")
            assertEquals(listOf("pubmed"), search.alerts().map { it.source })

            val group = curation.duplicates().single()
            assertEquals(null, group.duplicates.single().sharedIdentifier)
            val merged = curation.merge(group.reference.id, group.duplicates.single().reference.id)
            assertEquals(group.reference.id, merged.kept.id)
            assertTrue(curation.duplicates().isEmpty())
            assertTrue(curation.check().all { it.issues.isNotEmpty() })
        }
    }
}
