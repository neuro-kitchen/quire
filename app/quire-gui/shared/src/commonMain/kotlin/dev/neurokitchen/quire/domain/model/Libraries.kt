package dev.neurokitchen.quire.domain.model

/** A library; [name] `null` for an unregistered folder. */
data class LibraryEntry(val name: String?, val path: String, val registered: Boolean, val isDefault: Boolean)

data class BackupResult(val path: String, val files: Int, val bytes: Long)

/** What a sync session did: rows [applied], [kept] (older than what was here) or [refused] (dated ahead). */
data class SyncReport(
    val peer: String,
    val collaborator: String?,
    val joined: Boolean,
    val sent: Int,
    val applied: Int,
    val kept: Int,
    val refused: Int,
    val filesSent: Int,
    val filesReceived: Int,
)

/** A person a project is shared with: [role] `reader`, `editor` or `owner`. */
data class ProjectMember(val userId: String, val name: String, val role: String)
