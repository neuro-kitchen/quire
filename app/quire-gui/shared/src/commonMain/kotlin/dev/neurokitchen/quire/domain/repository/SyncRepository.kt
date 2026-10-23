package dev.neurokitchen.quire.domain.repository

import dev.neurokitchen.quire.domain.model.HubDevice
import dev.neurokitchen.quire.domain.model.HubLink
import dev.neurokitchen.quire.domain.model.LinkJoin
import dev.neurokitchen.quire.domain.model.LinkOffer
import dev.neurokitchen.quire.domain.model.UnlinkResult
import dev.neurokitchen.quire.domain.model.HubReport
import dev.neurokitchen.quire.domain.model.ProjectMember
import dev.neurokitchen.quire.domain.model.SyncReport

/**
 * Sync between the user's devices and sharing with collaborators, through the engine. Peers are
 * settings (`peers.<name>.id`, through [SettingsRepository]). Failures throw.
 */
interface SyncRepository {
    /** This device's ID: what the other devices add as a peer. */
    suspend fun deviceId(): String = unsupported()
    /** This person's user ID: what collaborators share projects with. */
    suspend fun userId(): String = unsupported()
    /** Offers to link a new device to this person's (its code to show; `.tasks/T14-security.md`). */
    suspend fun linkOffer(): LinkOffer = unsupported()
    /** Joins another device's code (`quire://device?…`), named [name] there. */
    suspend fun linkJoin(code: String, name: String): LinkJoin = unsupported()
    /** Syncs with [peer] (a peer's name from the settings, or a device ID). */
    suspend fun syncWith(peer: String): SyncReport = unsupported()
    /** Shares the project at [path] with [userId] (named [name]) as [role]: `reader`, `editor` or `owner`. */
    suspend fun shareProject(path: String, userId: String, name: String, role: String): ProjectMember = unsupported()
    suspend fun unshareProject(path: String, userId: String): Unit = unsupported()
    suspend fun projectMembers(path: String): List<ProjectMember> = unsupported()

    /** Reads a hub link (`quire://hub?…`); fails with what is wrong with it. */
    suspend fun parseHubLink(text: String): HubLink = unsupported()
    /** Syncs through the hub (setting `hub.id`); [code] registers this person the first time. */
    suspend fun syncWithHub(code: String? = null): HubReport = unsupported()
    /** This person's devices the hub knows. */
    suspend fun hubDevices(): List<HubDevice> = unsupported()
    /** Unlinks [device]: the hub refuses it and the library key rotates (it cannot read new data). */
    suspend fun unlinkHubDevice(device: String): UnlinkResult = unsupported()
    /** What to keep safe before syncing through a hub (it cannot recover the library). */
    suspend fun recoveryKit(): String = unsupported()
}
