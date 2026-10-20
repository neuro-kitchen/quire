package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.Creator
import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.Tag
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
import kotlin.test.assertTrue

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

    @Test
    fun addsEditsTrashesAndRestoresReferences() = runBlocking {
        withTimeout(30_000) {
            val repository = EngineLibraryRepository(EngineSession(open = { library() }))
            val typed = Reference(
                id = "", citeKey = "", kind = "article-journal", title = "Grid cells in the entorhinal cortex",
                creators = listOf(Creator(sortName = "Hafting", displayName = "Hafting, Torkel")),
                year = 2005, date = "2005-08", containerTitle = "Nature", doi = "10.1038/nature03721",
            )
            repository.addReference(typed)
            val added = repository.references(LibraryQuery(text = "entorhinal")).first().single()
            assertEquals("Hafting, Torkel", added.creators.single().displayName)
            assertEquals("10.1038/nature03721", added.doi)

            val edited = added.copy(volume = "436")
            assertEquals(listOf("volume"), repository.previewUpdate(edited).map { it.field })
            repository.updateReference(edited)
            assertEquals("436", repository.references(LibraryQuery(text = "entorhinal")).first().single().volume)

            repository.deleteReference(added.id)
            assertTrue(repository.references(LibraryQuery(text = "entorhinal")).first().isEmpty())
            assertEquals(1, repository.references(LibraryQuery(source = LibrarySource.Trash)).first().size)
            repository.restoreReference(added.id)
            assertEquals(1, repository.references(LibraryQuery(text = "entorhinal")).first().size)

            assertTrue(repository.issues(added.id).all { it.code.isNotBlank() && it.message.isNotBlank() })
            assertTrue(repository.failures().isEmpty())
        }
    }

    @Test
    fun organisesWithProjectsTagsAndRelations() = runBlocking {
        withTimeout(30_000) {
            val repository = EngineLibraryRepository(EngineSession(open = { library() }))
            repository.createProject("Thesis")
            assertTrue(repository.addToProject("buzsaki2006", "thesis"))
            assertEquals("phd-thesis", repository.renameProject("thesis", "PhD Thesis"))
            assertEquals(1, repository.projects().first { it.isNotEmpty() }.single().referenceCount)
            assertTrue(repository.removeFromProject("buzsaki2006", "phd-thesis"))
            repository.deleteProject("phd-thesis")
            assertTrue(repository.projects().first { it.isEmpty() }.isEmpty())

            repository.tag("buzsaki2006", listOf("rhythms", "brain"))
            repository.untag("buzsaki2006", listOf("brain"))
            assertEquals(Tag("rhythms", "#00ff00", 1), repository.colorTag("rhythms", "#00ff00"))
            assertEquals(listOf("rhythms"), repository.tags().first { it.isNotEmpty() }.map { it.name }.filter { it == "rhythms" })
            assertEquals(1, repository.deleteTag("rhythms"))

            assertTrue(repository.relate("hodgkin1952", "buzsaki2006"))
            assertEquals(listOf("hodgkin1952"), repository.related("buzsaki2006").map { it.citeKey })

            repository.addStandaloneNote("Reading plan")
            assertEquals(listOf("Reading plan"), repository.standaloneNotes().first { it.isNotEmpty() }.map { it.body })
        }
    }
}
