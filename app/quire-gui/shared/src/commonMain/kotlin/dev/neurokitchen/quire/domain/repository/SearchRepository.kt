package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.AddOutcome
import dev.neurokitchen.quire.domain.model.Alert
import dev.neurokitchen.quire.domain.model.AlertResult
import dev.neurokitchen.quire.domain.model.GraphKind
import dev.neurokitchen.quire.domain.model.GraphWork
import dev.neurokitchen.quire.domain.model.Reference
import dev.neurokitchen.quire.domain.model.SavedSearch
import dev.neurokitchen.quire.domain.model.SearchHit

/** Finding works, through the engine: search, saved searches, alerts, the citation graph. Failures throw. */
interface SearchRepository {
    /** The references `query` finds (the engine's search syntax), best first. */
    suspend fun search(query: String, limit: Int = 50): List<SearchHit> = unsupported()
    suspend fun saveSearch(name: String, query: String): SavedSearch = unsupported()
    suspend fun savedSearches(): List<SavedSearch> = unsupported()
    suspend fun runSaved(name: String, limit: Int = 50): List<SearchHit> = unsupported()
    suspend fun renameSaved(name: String, newName: String): SavedSearch = unsupported()
    suspend fun removeSaved(name: String): Unit = unsupported()

    /** [source]: `arxiv`, `pubmed` or `crossref`. */
    suspend fun addAlert(name: String, source: String, query: String): Alert = unsupported()
    suspend fun alerts(): List<Alert> = unsupported()
    suspend fun removeAlert(name: String): Unit = unsupported()
    /** Checks the alert `name` (or every alert) for new work, [days] back if given. */
    suspend fun checkAlerts(name: String? = null, days: Int? = null): List<AlertResult> = unsupported()
    /** Adds a work an alert found (from [AlertResult.found]) without a second lookup. */
    suspend fun addFound(work: Reference, source: String, projectPath: String? = null): AddOutcome = unsupported()

    /** The works next to a reference in the citation graph. */
    suspend fun graph(id: String, kind: GraphKind, limit: Int = 50): List<GraphWork> = unsupported()
}
