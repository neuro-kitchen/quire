package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.Alert
import dev.neurokitchen.quire.domain.model.AlertResult
import dev.neurokitchen.quire.domain.model.GraphKind
import dev.neurokitchen.quire.domain.model.GraphWork
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.SavedSearch
import dev.neurokitchen.quire.domain.model.SearchHit
import dev.neurokitchen.quire.domain.repository.SearchRepository
import dev.neurokitchen.quire.ffi.AlertRecord
import dev.neurokitchen.quire.ffi.SavedSearchRecord
import dev.neurokitchen.quire.ffi.SearchHitRecord

/** Search, saved searches, alerts and the citation graph through the engine (`quire-ffi`). */
class EngineSearchRepository(private val session: EngineSession) : SearchRepository {
    override suspend fun search(query: String, limit: Int) = session.library().search(query, limit.toUInt()).map { it.toDomain() }
    override suspend fun saveSearch(name: String, query: String) = session.library().saveSearch(name, query).toDomain()
    override suspend fun savedSearches() = session.library().savedSearches().map { it.toDomain() }
    override suspend fun runSaved(name: String, limit: Int) = session.library().runSaved(name, limit.toUInt()).map { it.toDomain() }
    override suspend fun renameSaved(name: String, newName: String) = session.library().renameSaved(name, newName).toDomain()
    override suspend fun removeSaved(name: String) = session.library().removeSaved(name)

    override suspend fun addAlert(name: String, source: String, query: String) = session.library().addAlert(name, source, query).toDomain()
    override suspend fun alerts() = session.library().alerts().map { it.toDomain() }
    override suspend fun removeAlert(name: String) = session.library().removeAlert(name)
    override suspend fun checkAlerts(name: String?, days: Int?) = session.library().checkAlerts(name, days?.toLong()).map { check ->
        AlertResult(check.alert.toDomain(), check.found.map { it.toPreview() }, check.error)
    }

    override suspend fun addFound(work: Reference, source: String, projectPath: String?) =
        session.library().addFound(work.toFields(), source, projectPath).toDomain()

    override suspend fun graph(id: String, kind: GraphKind, limit: Int) =
        session.library().graph(id, kind.name.lowercase(), limit.toUInt()).map {
            GraphWork(it.id, it.doi, it.title, it.year?.toInt(), it.firstAuthor, it.citedBy.toLong(), it.inLibrary)
        }

    private fun SearchHitRecord.toDomain() = SearchHit(reference.toDomain(), score, snippet, inText)
    private fun SavedSearchRecord.toDomain() = SavedSearch(name, query)
    private fun AlertRecord.toDomain() = Alert(name, source, query, checked)
}
