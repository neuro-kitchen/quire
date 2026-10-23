package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.BackupResult
import dev.neurokitchen.quire.domain.model.LibraryEntry
import dev.neurokitchen.quire.domain.repository.LibrariesRepository
import dev.neurokitchen.quire.ffi.LibraryEntryRecord
import dev.neurokitchen.quire.ffi.listLibraries
import dev.neurokitchen.quire.ffi.openLibrary
import dev.neurokitchen.quire.ffi.registerLibrary
import dev.neurokitchen.quire.ffi.renameLibrary
import dev.neurokitchen.quire.ffi.setDefaultLibrary
import dev.neurokitchen.quire.ffi.unregisterLibrary
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** Libraries in the config (`configFile` `null`: the usual one), opening another, backup, copies. */
class EngineLibrariesRepository(
    private val session: EngineSession,
    private val configFile: String? = null,
) : LibrariesRepository {
    private suspend fun <T> io(block: () -> T): T = withContext(Dispatchers.IO) { block() }

    override suspend fun libraries() = io { listLibraries(configFile).map { it.toDomain() } }
    override suspend fun open(name: String?) = session.switchTo { openLibrary(name, configFile) }
    override suspend fun register(name: String, path: String?) = io { registerLibrary(configFile, name, path).toDomain() }
    override suspend fun setDefault(name: String) = io { setDefaultLibrary(configFile, name).toDomain() }
    override suspend fun rename(name: String, newName: String) = io { renameLibrary(configFile, name, newName).toDomain() }
    override suspend fun unregister(name: String) = io { unregisterLibrary(configFile, name).toDomain() }

    override suspend fun backup(output: String): BackupResult {
        val report = session.library().backup(output)
        return BackupResult(report.path, report.files.toInt(), report.bytes.toLong())
    }

    override suspend fun copyTo(id: String, library: String, projectPath: String?) =
        session.library().copyToLibrary(id, library, projectPath).toDomain()

    private fun LibraryEntryRecord.toDomain() = LibraryEntry(name, path, registered, isDefault)
}
