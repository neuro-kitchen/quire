package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.LibrarySource
import dev.neurokitchen.quire.domain.model.SortColumn
import dev.neurokitchen.quire.ffi.QuireLibrary
import dev.neurokitchen.quire.ffi.openLibraryAt
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals

/** The real engine through the generated bindings and the native library on the classpath. */
class EngineLibraryRepositoryTest {
    private val bib = """
        @article{hodgkin1952, title = {A quantitative description of membrane current}, author = {Hodgkin, A. L. and Huxley, A. F.},
          journal = {The Journal of Physiology}, year = {1952}, doi = {10.1113/jphysiol.1952.sp004764}}
        @book{buzsaki2006, title = {Rhythms of the Brain}, author = {Buzsáki, György}, publisher = {Oxford University Press}, year = {2006}}
    """.trimIndent()

    private suspend fun library(): QuireLibrary {
        val dir = Files.createTempDirectory("quire-test")
        val library = openLibraryAt(dir.resolve("lib").toString(), dir.resolve("config.toml").toString())
        val file = dir.resolve("refs.bib").also { Files.writeString(it, bib) }
        assertEquals(2u, library.importBib(file.toString(), null))
        return library
    }

    @Test
    fun listsTheLibraryAndFollowsChanges() = runBlocking {
        withTimeout(30_000) {
            val engine = library()
            val repository = EngineLibraryRepository(EngineSession(open = { engine }))
            val references = repository.references(LibraryQuery(sort = SortColumn.Year))
            val first = references.first()
            assertEquals(listOf("hodgkin1952", "buzsaki2006"), first.map { it.citeKey })
            assertEquals("Huxley, A. F.", first[0].creators[1].displayName)
            assertEquals("The Journal of Physiology", first[0].containerTitle)

            // A project made through the engine reaches an open projects flow.
            engine.createProject("Thesis", null)
            assertEquals(listOf("thesis"), repository.projects().first { it.isNotEmpty() }.map { it.path })
            assertEquals(
                listOf("buzsaki2006"),
                repository.references(LibraryQuery(source = LibrarySource.Unfiled, text = "rhythms")).first().map { it.citeKey },
            )
        }
    }
}
