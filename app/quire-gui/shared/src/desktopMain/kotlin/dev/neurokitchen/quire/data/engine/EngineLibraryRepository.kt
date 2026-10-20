package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.Note
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.Tag
import dev.neurokitchen.quire.domain.repository.LibraryDescription
import dev.neurokitchen.quire.domain.repository.LibraryRepository

/** The library's references and projects through the engine (`quire-ffi`): reading, adding, editing. */
class EngineLibraryRepository(private val session: EngineSession) : LibraryRepository {
    override suspend fun describe(): LibraryDescription {
        val info = session.library().info()
        return LibraryDescription(name = info.name ?: info.path, path = info.path, configFile = info.configFile)
    }

    override fun references(query: LibraryQuery) =
        session.reloading { it.references(query.toFfi()).map { record -> record.toDomain() } }

    override fun projects() = session.reloading { it.projects().map { record -> record.toDomain() } }

    override suspend fun reopen() = session.reopen()

    override suspend fun addReference(reference: Reference) =
        session.library().addManual(reference.toFields(), reference.projects.firstOrNull()).toDomain()

    override suspend fun updateReference(reference: Reference) {
        session.library().edit(reference.id, reference.toFields(), null, false)
    }

    override suspend fun previewUpdate(reference: Reference) =
        session.library().edit(reference.id, reference.toFields(), null, true).map { it.toDomain() }

    override suspend fun deleteReference(id: String) = session.library().trash(id)
    override suspend fun restoreReference(id: String) = session.library().restore(id)

    override suspend fun createProject(name: String, parentPath: String?) = session.library().createProject(name, parentPath).path

    override suspend fun lookup(input: String) = session.library().lookup(input).toPreview()

    override suspend fun addByIdentifier(input: String, projectPath: String?) =
        session.library().add(input, projectPath).toDomain()

    override suspend fun addFromPage(url: String, html: String, projectPath: String?) =
        session.library().addFromPage(url, html, projectPath).toDomain()

    override suspend fun refetch(id: String, dryRun: Boolean) = session.library().refetch(id, dryRun).map { it.toDomain() }
    override suspend fun issues(id: String) = session.library().issues(id).map { it.toDomain() }
    override suspend fun failures() = session.library().failures().map { it.toDomain() }
    override suspend fun forgetFailure(input: String) = session.library().forgetFailure(input)

    override fun tags() = session.reloading { library -> library.tags().map { Tag(it.name, it.color, it.count.toInt()) } }
    override suspend fun tag(id: String, names: List<String>) = session.library().tag(id, names)
    override suspend fun untag(id: String, names: List<String>) = session.library().untag(id, names)
    override suspend fun renameTag(old: String, new: String) = session.library().renameTag(old, new)
    override suspend fun deleteTag(name: String) = session.library().deleteTag(name).toInt()
    override suspend fun colorTag(name: String, color: String?, position: Int?): Tag {
        val tag = session.library().colorTag(name, color, position?.toUByte())
        return Tag(tag.name, tag.color, tag.count.toInt())
    }

    override suspend fun deleteProject(path: String) {
        session.library().removeProject(path, true)
    }

    override suspend fun renameProject(path: String, name: String) = session.library().renameProject(path, name).path
    override suspend fun addToProject(id: String, projectPath: String) = session.library().link(id, projectPath)
    override suspend fun removeFromProject(id: String, projectPath: String) = session.library().unlink(id, projectPath)

    override suspend fun related(id: String) = session.library().related(id).map { it.toDomain() }
    override suspend fun relate(a: String, b: String) = session.library().relate(a, b)
    override suspend fun unrelate(a: String, b: String) = session.library().unrelate(a, b)

    override fun standaloneNotes() = session.reloading { library ->
        library.standaloneNotes().map { Note(it.id, it.body, it.updated) }
    }

    override suspend fun addStandaloneNote(body: String) {
        session.library().addStandaloneNote(body)
    }
}
