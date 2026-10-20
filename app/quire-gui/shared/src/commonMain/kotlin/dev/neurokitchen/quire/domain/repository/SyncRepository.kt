package dev.neurokitchen.quire.domain.repository

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
    /** The user key, to give another of the user's devices (keep it secret). */
    suspend fun exportUserKey(): String = unsupported()
    /** Makes another device's exported key this device's; returns the user ID. */
    suspend fun importUserKey(key: String): String = unsupported()
    /** Syncs with [peer] (a peer's name from the settings, or a device ID). */
    suspend fun syncWith(peer: String): SyncReport = unsupported()
    /** Shares the project at [path] with [userId] (named [name]) as [role]: `reader`, `editor` or `owner`. */
    suspend fun shareProject(path: String, userId: String, name: String, role: String): ProjectMember = unsupported()
    suspend fun unshareProject(path: String, userId: String): Unit = unsupported()
    suspend fun projectMembers(path: String): List<ProjectMember> = unsupported()
}
