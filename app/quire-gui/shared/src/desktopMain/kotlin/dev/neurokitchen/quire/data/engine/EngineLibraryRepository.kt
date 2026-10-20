package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.repository.LibraryDescription
import dev.neurokitchen.quire.domain.repository.LibraryRepository

/** The library's references and projects through the engine (`quire-ffi`). */
class EngineLibraryRepository(private val session: EngineSession) : LibraryRepository {
    override suspend fun describe(): LibraryDescription {
        val info = session.library().info()
        return LibraryDescription(name = info.name ?: info.path, path = info.path, configFile = info.configFile)
    }

    override fun references(query: LibraryQuery) =
        session.reloading { it.references(query.toFfi()).map { record -> record.toDomain() } }

    override fun projects() = session.reloading { it.projects().map { record -> record.toDomain() } }

    override suspend fun reopen() = session.reopen()
}
