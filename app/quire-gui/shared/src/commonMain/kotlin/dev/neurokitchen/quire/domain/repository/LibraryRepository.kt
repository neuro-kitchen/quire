package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.LibraryQuery
import dev.neurokitchen.quire.domain.model.Project
import dev.neurokitchen.quire.domain.model.Reference
import kotlinx.coroutines.flow.Flow

/**
 * The library, as the screens need it. Rules (search, sorting, merging, validation) belong to the
 * Rust engine; implementations only translate its calls and change stream. Flows emit again after
 * every change to the library and fail with the engine's message.
 */
interface LibraryRepository {
    /** The open library's name and folder, for titles. */
    suspend fun describe(): LibraryDescription

    /** References answering `query`. */
    fun references(query: LibraryQuery): Flow<List<Reference>>

    /** Every project, parents before their children. */
    fun projects(): Flow<List<Project>>

    /** Opens the library again, e.g. after its settings changed; open flows switch to it. */
    suspend fun reopen()

    suspend fun addReference(reference: Reference): Unit = Unit
    suspend fun updateReference(reference: Reference): Unit = Unit
    suspend fun deleteReference(id: String): Unit = Unit
    suspend fun createProject(name: String, parentPath: String? = null): Unit = Unit
    suspend fun deleteProject(path: String): Unit = Unit
}

data class LibraryDescription(val name: String, val path: String, val configFile: String)
