package dev.neurokitchen.quire.data.engine

import dev.neurokitchen.quire.domain.model.HubDevice
import dev.neurokitchen.quire.domain.model.HubLink
import dev.neurokitchen.quire.domain.model.LinkJoin
import dev.neurokitchen.quire.domain.model.LinkOffer
import dev.neurokitchen.quire.domain.model.LinkRequest
import dev.neurokitchen.quire.domain.model.HubReport
import dev.neurokitchen.quire.domain.model.ProjectMember
import dev.neurokitchen.quire.domain.model.SyncReport
import dev.neurokitchen.quire.domain.repository.SyncRepository
import dev.neurokitchen.quire.ffi.MemberRecord
import dev.neurokitchen.quire.ffi.parseHubLink as ffiParseHubLink

/** Sync and sharing through the engine (`quire-ffi`). */
class EngineSyncRepository(private val session: EngineSession) : SyncRepository {
    override suspend fun deviceId() = session.library().deviceId()
    override suspend fun userId() = session.library().userId()

    override suspend fun linkOffer(): LinkOffer {
        val offer = session.library().linkOffer()
        return object : LinkOffer {
            override val uri = offer.uri()
            override suspend fun nextRequest() = offer.nextRequest().let { LinkRequest(it.name, it.words) }
            override suspend fun allow() = offer.allow()
            override suspend fun refuse() = offer.refuse()
            override suspend fun cancel() = offer.cancel()
        }
    }

    override suspend fun linkJoin(code: String, name: String): LinkJoin {
        val join = session.library().linkJoin(code, name)
        return object : LinkJoin {
            override val words = join.words()
            override suspend fun finish() = join.finish()
            override suspend fun cancel() = join.cancel()
        }
    }

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

    override suspend fun parseHubLink(text: String) = ffiParseHubLink(text).let { HubLink(it.id, it.relay, it.code) }

    override suspend fun syncWithHub(code: String?): HubReport {
        val r = session.library().syncWithHub(code)
        return HubReport(
            r.registered, r.joined, r.pushed.toInt(), r.applied.toInt(), r.filesUploaded.toInt(), r.filesDownloaded.toInt(),
            r.overQuota.toInt(), r.used.toLong(), r.quota.toLong(),
        )
    }

    override suspend fun hubDevices() =
        session.library().hubDevices().map { HubDevice(it.device, it.firstSeen, it.lastSeen, it.revoked) }

    override suspend fun revokeHubDevice(device: String) = session.library().hubRevoke(device)
    override suspend fun recoveryKit() = session.library().recoveryKit()

    private fun MemberRecord.toDomain() = ProjectMember(user, name, role)
}
