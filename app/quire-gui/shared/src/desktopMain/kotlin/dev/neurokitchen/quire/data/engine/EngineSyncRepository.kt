package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.ProjectMember
import dev.neurokitchen.quire.domain.model.SyncReport
import dev.neurokitchen.quire.domain.repository.SyncRepository
import dev.neurokitchen.quire.ffi.MemberRecord

/** Sync and sharing through the engine (`quire-ffi`). */
class EngineSyncRepository(private val session: EngineSession) : SyncRepository {
    override suspend fun deviceId() = session.library().deviceId()
    override suspend fun userId() = session.library().userId()
    override suspend fun exportUserKey() = session.library().exportUserKey()
    override suspend fun importUserKey(key: String) = session.library().importUserKey(key)

    override suspend fun syncWith(peer: String): SyncReport {
        val r = session.library().syncWith(peer)
        return SyncReport(
            r.peer, r.collaborator, r.joined, r.sent.toInt(), r.applied.toInt(), r.kept.toInt(), r.refused.toInt(),
            r.filesSent.toInt(), r.filesReceived.toInt(),
        )
    }

    override suspend fun shareProject(path: String, userId: String, name: String, role: String) =
        session.library().shareProject(path, userId, name, role).toDomain()

    override suspend fun unshareProject(path: String, userId: String) = session.library().unshareProject(path, userId)
    override suspend fun projectMembers(path: String) = session.library().projectMembers(path).map { it.toDomain() }

    private fun MemberRecord.toDomain() = ProjectMember(user, name, role)
}
