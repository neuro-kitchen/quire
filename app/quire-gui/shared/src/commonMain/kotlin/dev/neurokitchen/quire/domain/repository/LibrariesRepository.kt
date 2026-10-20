package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.AddOutcome
import dev.neurokitchen.quire.domain.model.BackupResult
import dev.neurokitchen.quire.domain.model.LibraryEntry

/** The libraries in the config, opening another, backup, copying between them. Failures throw. */
interface LibrariesRepository {
    /** The libraries the config knows, the default first. */
    suspend fun libraries(): List<LibraryEntry> = unsupported()
    /** Opens the library [name] (`null`: the default); every repository and open flow switches to it. */
    suspend fun open(name: String?): Unit = unsupported()
    /** Registers a library [name] in [path] (`null`: the usual place). */
    suspend fun register(name: String, path: String? = null): LibraryEntry = unsupported()
    suspend fun setDefault(name: String): LibraryEntry = unsupported()
    suspend fun rename(name: String, newName: String): LibraryEntry = unsupported()
    /** Forgets a registered library (its folder stays). */
    suspend fun unregister(name: String): LibraryEntry = unsupported()
    /** Writes the open library to [output] (`.tar.gz`, readable by the user only). */
    suspend fun backup(output: String): BackupResult = unsupported()
    /** Copies a reference into the library [library] (keeping its ID), into a project there if given. */
    suspend fun copyTo(id: String, library: String, projectPath: String? = null): AddOutcome = unsupported()
}
