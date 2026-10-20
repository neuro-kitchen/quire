package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.CheckedReference
import dev.neurokitchen.quire.domain.model.DuplicateGroup
import dev.neurokitchen.quire.domain.model.MergeResult
import dev.neurokitchen.quire.domain.model.RetractionResult

/** Keeping the library clean, through the engine: duplicates, checks, retractions. Failures throw. */
interface CurationRepository {
    suspend fun duplicates(): List<DuplicateGroup> = unsupported()
    /** Merges [other] into [keep]: its fields, projects, tags, notes, PDFs and highlights move over. */
    suspend fun merge(keep: String, other: String): MergeResult = unsupported()
    /** Records that [a] and [b] are distinct works; `false` if already recorded. */
    suspend fun dismissDuplicate(a: String, b: String): Boolean = unsupported()
    /** References with issues in the project at [projectPath] (or the library). */
    suspend fun check(projectPath: String? = null, recursive: Boolean = true): List<CheckedReference> = unsupported()
    /** Retraction and correction notices for references with a DOI (online). */
    suspend fun checkRetractions(projectPath: String? = null, recursive: Boolean = true): List<RetractionResult> = unsupported()
}
