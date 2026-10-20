package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.CheckedReference
import dev.neurokitchen.quire.domain.model.Duplicate
import dev.neurokitchen.quire.domain.model.DuplicateGroup
import dev.neurokitchen.quire.domain.model.MergeResult
import dev.neurokitchen.quire.domain.model.Notice
import dev.neurokitchen.quire.domain.model.RetractionResult
import dev.neurokitchen.quire.domain.repository.CurationRepository

/** Duplicates, checks and retractions through the engine (`quire-ffi`). */
class EngineCurationRepository(private val session: EngineSession) : CurationRepository {
    override suspend fun duplicates() = session.library().duplicates().map { group ->
        DuplicateGroup(group.reference.toDomain(), group.duplicates.map { Duplicate(it.reference.toDomain(), it.identifier) })
    }

    override suspend fun merge(keep: String, other: String): MergeResult {
        val m = session.library().merge(keep, other)
        return MergeResult(
            m.kept.toDomain(), m.removedId, m.fields, m.projects.toInt(), m.tags.toInt(), m.notes.toInt(),
            m.attachments.toInt(), m.highlights.toInt(), m.relations.toInt(),
        )
    }

    override suspend fun dismissDuplicate(a: String, b: String) = session.library().dismissDuplicate(a, b)

    override suspend fun check(projectPath: String?, recursive: Boolean) = session.library().check(projectPath, recursive).map {
        CheckedReference(it.id, it.citeKey, it.issues.map { issue -> issue.toDomain() })
    }

    override suspend fun checkRetractions(projectPath: String?, recursive: Boolean) =
        session.library().checkRetractions(projectPath, recursive).map { check ->
            RetractionResult(check.reference.toDomain(), check.notices.map { Notice(it.kind, it.notice, it.date, it.source) }, check.error)
        }
}
